//! Process details via `sysinfo`. One `System` lives in `AppState` so CPU usage can be diffed.

use std::collections::HashMap;

use serde::Serialize;
use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcInfo {
    pub name: String,
    pub exe: Option<String>,
    pub cmdline: Option<String>,
    pub cwd: Option<String>,
    pub parent_pid: Option<u32>,
    pub parent_name: Option<String>,
    /// Share of total CPU capacity, 0–100.
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub disk_read_bytes: u64,
    pub disk_written_bytes: u64,
    pub start_time_ms: i64,
    /// False when details could not be read (other user / elevated process).
    pub accessible: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcRef {
    pub pid: u32,
    pub name: String,
    pub exe: Option<String>,
}

/// How much to read per process. `Light` is for the tray-only background mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detail {
    /// Name, start time, parent, exe, cmdline, cwd, CPU, memory, disk I/O.
    Full,
    /// Name, start time, parent, exe (+ cwd for socket owners, see `refresh_pids`).
    Light,
}

fn refresh_kind(detail: Detail) -> ProcessRefreshKind {
    let base = ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet);
    match detail {
        Detail::Light => base,
        Detail::Full => base
            .with_cpu()
            .with_memory()
            .with_disk_usage()
            .with_cmd(UpdateKind::OnlyIfNotSet)
            .with_cwd(UpdateKind::OnlyIfNotSet),
    }
}

/// Refreshes only `pids` and their direct parents — far cheaper than walking every process.
pub fn refresh_pids(sys: &mut System, pids: &[u32], detail: Detail) {
    let mut list: Vec<Pid> = pids.iter().map(|p| Pid::from_u32(*p)).collect();
    list.sort_unstable();
    list.dedup();
    // Socket owners also get their cwd in Light mode (once per process) so background history
    // events carry the project name; `refresh_all_light` (all processes) stays cwd-free.
    let kind = match detail {
        Detail::Light => refresh_kind(detail).with_cwd(UpdateKind::OnlyIfNotSet),
        Detail::Full => refresh_kind(detail),
    };
    sys.refresh_processes_specifics(ProcessesToUpdate::Some(&list), true, kind);

    let mut parents: Vec<Pid> = list
        .iter()
        .filter_map(|p| sys.process(*p)?.parent())
        .filter(|p| list.binary_search(p).is_err())
        .collect();
    parents.sort_unstable();
    parents.dedup();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&parents),
        true,
        refresh_kind(Detail::Light),
    );
}

/// Refreshes every process with minimal data; needed for process trees (details, tree kill).
pub fn refresh_all_light(sys: &mut System) {
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind(Detail::Light));
}

fn name_of(p: &Process) -> String {
    p.name().to_string_lossy().into_owned()
}

fn cmdline_of(p: &Process) -> Option<String> {
    let args: Vec<String> = p
        .cmd()
        .iter()
        .map(|a| {
            let a = a.to_string_lossy();
            if a.contains(' ') {
                format!("\"{a}\"")
            } else {
                a.into_owned()
            }
        })
        .collect();
    (!args.is_empty()).then(|| args.join(" "))
}

/// The parent is only trusted if it started before the child (guards against PID reuse).
fn real_parent<'a>(sys: &'a System, p: &Process) -> Option<&'a Process> {
    let parent = sys.process(p.parent()?)?;
    (parent.start_time() <= p.start_time()).then_some(parent)
}

pub fn lookup(sys: &System, pid: u32, cpu_count: f32) -> Option<ProcInfo> {
    let p = sys.process(Pid::from_u32(pid))?;
    let parent = real_parent(sys, p);
    let exe = p.exe().map(|e| e.to_string_lossy().into_owned());
    let disk = p.disk_usage();
    Some(ProcInfo {
        name: name_of(p),
        accessible: exe.is_some(),
        exe,
        cmdline: cmdline_of(p),
        cwd: p
            .cwd()
            .map(|c| c.to_string_lossy().into_owned())
            .filter(|c| !c.is_empty()),
        parent_pid: parent.map(|pp| pp.pid().as_u32()),
        parent_name: parent.map(name_of),
        cpu_percent: p.cpu_usage() / cpu_count.max(1.0),
        memory_bytes: p.memory(),
        disk_read_bytes: disk.total_read_bytes,
        disk_written_bytes: disk.total_written_bytes,
        start_time_ms: p.start_time() as i64 * 1000,
    })
}

/// Ancestors from the direct parent upwards (max 8 levels).
pub fn parent_chain(sys: &System, pid: u32) -> Vec<ProcRef> {
    let mut chain = Vec::new();
    let Some(mut cur) = sys.process(Pid::from_u32(pid)) else {
        return chain;
    };
    while let Some(parent) = real_parent(sys, cur) {
        chain.push(ProcRef {
            pid: parent.pid().as_u32(),
            name: name_of(parent),
            exe: parent.exe().map(|e| e.to_string_lossy().into_owned()),
        });
        if chain.len() >= 8 {
            break;
        }
        cur = parent;
    }
    chain
}

/// All descendants of `pid`, deepest first (safe kill order).
pub fn descendants(sys: &System, pid: u32) -> Vec<(u32, String)> {
    let mut children: HashMap<u32, Vec<&Process>> = HashMap::new();
    for p in sys.processes().values() {
        if let Some(parent) = real_parent(sys, p) {
            children.entry(parent.pid().as_u32()).or_default().push(p);
        }
    }
    let mut out = Vec::new();
    fn walk(pid: u32, children: &HashMap<u32, Vec<&Process>>, out: &mut Vec<(u32, String)>) {
        for c in children.get(&pid).into_iter().flatten() {
            walk(c.pid().as_u32(), children, out);
            out.push((c.pid().as_u32(), name_of(c)));
        }
    }
    walk(pid, &children, &mut out);
    out
}

pub fn cpu_count() -> f32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as f32)
        .unwrap_or(1.0)
}
