# Peron — Cam tasarımını Windows'a taşıma rehberi

Dal: `feat/macos-support`. Bu rehber, macOS'ta tamamlanan cam (Liquid Glass) arayüzünün
**Windows 11'e** uyarlanması içindir ve Windows bilgisayarda uygulanacak. Linux etkilenmeyecek.

## 1. Bağlam: macOS'ta ne yapıldı

| Commit | İçerik |
|---|---|
| `cc7ccde` | Yerel cam katmanı. `src-tauri/src/glass.rs` WebView'in arkasına `NSGlassEffectView` koyuyor. Pencere saydam, başlık çubuğu yok. |
| `d2303fc` | `src/styles.macos.css`: yalnızca `[data-platform="macos"]` altında geçerli tasarım. |
| `46c7170` | Tasarım ayrıntıları: ayrı yüzen cam kapsüller, renk tonlu seçim, arama kapsülü. |

Tasarımın katmanları şöyle; Windows'ta da **aynen korunacak**:

1. **Pencere:** Arkada sistem malzemesi var, WebView saydam.
2. **Krom** (araç çubuğu, filtre çubuğu, durum çubuğu): Kendi arka planı yok, malzeme görünür.
3. **İçerik** (port tablosu, detay, geçmiş, iletişim): Okunabilirlik için yuvarlak köşeli (12px),
   yarı opak paneller. **Cam asla içeriğe uygulanmaz.**
4. **Yüzen öğeler** (menü, diyalog, bildirim): CSS camı.
   - `backdrop-filter: blur() saturate()`
   - üstte 1px parlak kenar
   - yumuşak gölge
5. **Kontroller:**
   - Sekmeler, görünüm seçici, segmentli kontroller ve filtre çipleri **ayrı yüzen cam kapsüller**dir.
   - Seçili olan **renk tonlu cam**dır: vurgu rengi %82, üstte beyaz parlama gradyanı, iç kenar
     ve vurgu renginde gölge.
   - Arama kutusu tek bir cam kapsüldür; içindeki input çıplaktır.

## 2. Windows karşılıkları ve kararlar

| macOS | Windows 11 | Windows 10 |
|---|---|---|
| `NSGlassEffectView` (pencere camı) | **Mica**. Tauri'nin kendi API'si: `Effect::Mica`. | Efekt yok, opak pencere. Acrylic, Win10'da sürükleme/boyutlandırmada takılıyor. |
| Başlık çubuğu gizli, trafik ışıkları araç çubuğunda | **Yerel başlık çubuğu kalır.** Mica onu da kaplar; tema `theme.ts` → `setTheme` ile zaten eşleniyor. | Aynı |
| SF Pro / SF Mono | Segoe UI Variable / Cascadia Mono. `styles.css`'te zaten var. | Segoe UI |
| Sistem vurgu rengi (`AccentColor` CSS) | Rust'tan okunur (§4.3). WebView2'de `AccentColor` güvenilir değil. | Aynı |
| Reduce Transparency / Increase Contrast | "Saydamlık efektleri" kapalıysa Mica'yı sistem kapatır. Yüksek karşıtlıkta `forced-colors` (§4.4). | Aynı |

**Görsel dil iki platformda aynı kalır:** kapsüller, renk tonlu seçim ve içerik panelleri. Yalnızca
yazı tipi, vurgu rengi kaynağı ve pencere malzemesi değişir. Fluent'in 4px köşeli kontrollerine
geçilmez; marka tutarlılığı için kapsül dili korunur.

## 3. Başlarken (Windows makinesi)

```powershell
git fetch origin
git switch feat/macos-support      # ya da: git switch -c feat/macos-support --track origin/feat/macos-support
git pull
npm ci
cd src-tauri; cargo test --workspace; cd ..
npm test
npm run tauri dev
```

Ön koşullar `dev-run` skill'inde anlatılıyor (MSVC, WebView2).

`.claude/skills/macos-hig-designer` bu makineye gelmez, çünkü `.claude/` yerelde git dışında. Bu iş
için gerekmiyor: tasarım kuralları bu rehberde ve `styles.macos.css` başındaki açıklamada.

## 4. Uygulama adımları

### 4.1 Stil dosyasını paylaşılır hale getir

Amaç: aynı kuralların iki kopyası olmasın; macOS görünümü **piksel piksel aynı** kalsın.

1. `src/styles.macos.css` dosyasını `git mv` ile `src/styles.glass.css` olarak yeniden adlandır.
   `src/main.tsx` içindeki import'u güncelle.
2. Platformdan bağımsız kuralların seçicilerini `[data-platform="macos"]` yerine `[data-glass]` yap:
   - token'lar,
   - krom,
   - içerik panelleri,
   - kapsüller,
   - yüzen öğeler,
   - erişilebilirlik.
3. Platforma özgü kalanlar kendi bloklarına gider:
   - `[data-platform="macos"]`:
     - SF yazı tipleri ve `-webkit-font-smoothing`;
     - `.toolbar` sol boşluğu (86px, trafik ışıkları);
     - `--accent: AccentColor` bloğu.
   - `[data-platform="windows"]`:
     - yazı tipi yok, `styles.css`'teki Segoe yığını kalır;
     - `.toolbar` normal boşluk (`padding: 8px 14px`);
     - `min-height: 48px`.
4. `src/platform.ts`:
   - `markPlatform()` `data-platform` değerini `macos` / `windows` / `linux` olarak koysun.
   - `data-glass` yalnızca `macos` ve `windows` için konsun.
   - Windows tespiti `/win/i` ile `navigator.platform || navigator.userAgent` üzerinden yapılır.
   - `src/platform.test.ts`'e şu testi ekle: Windows'ta `data-glass` var, Linux'ta yok.
5. `dragRegion()` macOS'a özgü kalır. Windows'ta başlık çubuğu var, sürükleme alanı gerekmez.

### 4.2 Yerel malzeme: Mica

- **`src-tauri/tauri.windows.conf.json`** (yeni):
  - `tauri.macos.conf.json` gibi `app.windows` dizisini **tam** yaz. JSON merge patch dizileri
    birleştirmez, değiştirir.
  - `main` penceresinin bütün alanlarını kopyala, `transparent: true` ekle.
  - `decorations` varsayılan (true) kalsın.
  - `bundle` ayarlarına dokunma; NSIS ayarları ana dosyada.
- **`src-tauri/src/tray.rs` → `apply_glass`:** Mevcut `#[cfg(not(target_os = "macos"))]` dalını
  ikiye ayır, Windows dalı şöyle:
  ```rust
  #[cfg(windows)]
  {
      use tauri::window::{Effect, EffectsBuilder};
      if crate::platform::is_windows_11() {
          if let Err(e) = window.set_effects(EffectsBuilder::new().effect(Effect::Mica).build()) {
              crate::log_error!("mica unavailable: {e}");
              let _ = window.set_background_color(Some(tauri::window::Color(243, 243, 243, 255)));
          }
      } else {
          // Windows 10: no Mica; Acrylic lags on drag/resize. Stay opaque.
          let _ = window.set_background_color(Some(tauri::window::Color(243, 243, 243, 255)));
      }
  }
  ```
  - Opak yedek renk koyu temada `(32, 32, 32, 255)` olmalı.
  - Mevcut tema: `settings.theme.window_theme()`, ya da sistem teması için `window.theme()`.
- **`crates/core/src/platform/windows.rs`:** `is_windows_11()` ekle; build ≥ 22000 Windows 11
  demektir.
  - Sürüm bilgisi zaten `sysinfo::System::os_version()` ile alınabiliyor (`platform::os_description`).
  - `RtlGetVersion` gerekiyorsa `win32-netapi` skill'indeki kurallara uy (`unsafe` yalnızca core'da).
- **Ek bağımlılık gerekmez.** Tauri'nin içindeki `window-vibrancy` 0.6 Mica'yı destekliyor.
  - macOS'ta `window-vibrancy` 0.7 eklemek Tauri'nin 0.6'sıyla bağlama hatası verdi
    (`NSVisualEffectViewTagged`). Windows'ta da ikinci bir sürüm **eklenmemeli**.
- `macOSPrivateApi` / `macos-private-api` ana yapılandırmada açık. Bunlar Windows'ta etkisiz;
  `tauri-build` platformdan bağımsız kontrol ettiği için orada durmaları gerekiyor. Dokunma.

### 4.3 Sistem vurgu rengi

- **Rust:**
  - `platform::accent_color() -> Option<String>` (`#rrggbb`) ekle. Kaynak:
    `windows::UI::ViewManagement::UISettings::GetColorValue(UIColorType::Accent)`.
  - Gerekli `windows` crate özelliği: `UI_ViewManagement`. Bunu `crates/core/Cargo.toml`'daki
    Windows özellik listesine ekle.
  - `AppInfo`'ya (`src-tauri/src/commands.rs`) `accent: Option<String>` alanını ekle.
  - Diğer platformlarda değer `None`.
- **UI:**
  - `src/types.ts`'teki `AppInfo`'ya `accent?: string | null` ekle.
  - `App.tsx`'te info yüklenince, değer varsa
    `document.documentElement.style.setProperty("--accent-fallback", info.accent)` çağır.
    - `--accent` zaten `var(--accent-fallback)` üzerinden çözülüyor, başka değişiklik gerekmez.
    - Koyu temada aynı renk kullanılabilir. Windows açık vurgu tonunu da veriyor:
      `UIColorType::AccentLight1`; istenirse koyu tema için kullanılır.
- Vurgu rengi uygulama çalışırken değişirse sonraki açılışta güncellenir. Canlı takip şart değil.

### 4.4 Yüksek karşıtlık (forced colors): zorunlu

Peron Store'da ve yüksek karşıtlık testi geçmiş durumda (`docs/MANUAL_TESTS.md` #3). Bu
korunmalı:

- `[data-glass]` kurallarının sonuna bir `@media (forced-colors: active)` bloğu ekle. Bu blok:
  - bütün `backdrop-filter`'ları ve yarı saydam arka planları kaldırır;
  - kapsüllere `border: 1px solid ButtonText` verir;
  - seçili kapsüllerde `background: Highlight; color: HighlightText` kullanır.
    `styles.css`'in sonundaki mevcut `forced-colors` bloğu örnek alınabilir.
- Yüksek karşıtlıkta Mica'yı uygulama: §4.2'deki dalda
  `UISettings`/`SystemParametersInfo(SPI_GETHIGHCONTRAST)` kontrolüyle opak arka plana düş.

### 4.5 Bilinen tuzaklar (macOS'ta yaşananlar)

- **Genel düğme kuralı tablo başlıklarına bulaşır.** Başlıklardaki `.th-sort` düğmeleri `all: unset`
  ile stilsizdir. Kapsül kuralı `button:not(.th-sort)` olarak kalmalı.
- **Genel `input[type="search"]` kuralı `.search` kapsülünün içindeki input'a ikinci kutu çizer.**
  Bağımsız aramalar için `:not(.search input)` kullanılıyor.
- **Seçili kapsülün okunabilirliği:** Açık temada beyaz yazı, %82 vurgu üzerinde. Windows'un bazı
  açık vurgu renklerinde (sarı, açık yeşil) karşıtlık düşebilir. Vurgunun parlaklığına göre metni
  koyulaştıran bir kural gerekebilir; kontrol et (§5).

## 5. Doğrulama

**Otomatik:**
```powershell
npx tsc --noEmit; npm test
cd src-tauri; cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace -- --include-ignored
```

**Elle (Windows 11):**
- [ ] Mica görünüyor: masaüstü renkleri pencere zeminine hafifçe yansıyor.
- [ ] Başlık çubuğu açık/koyu temaya uyuyor. Ayarlar > Tema ile anlık geçiş çalışıyor.
- [ ] Kapsüller ve seçili kapsül macOS'takiyle aynı; vurgu rengi Windows vurgu rengiyle aynı.
  Ayarlar > Kişiselleştirme > Renkler'den sarı ve yeşil vurgu dahil 2–3 renk denenmeli.
- [ ] Menü, diyalog ve bildirim kutusu cam görünümünde; tablo metni okunaklı.
- [ ] "Saydamlık efektleri" kapalıyken pencere opak ve düzgün.
- [ ] Yüksek karşıtlık açıkken (Aquatic/Desert) her öğe görünür, seçim belli
  (`MANUAL_TESTS.md` #3'ü yeniden yap).
- [ ] Tepsiye kapat → yeniden aç: Mica tekrar uygulanıyor (pencere her açılışta yeniden yaratılıyor).
- [ ] `release-build` skill'i ile NSIS kur. MSIX'i derle, kur ve WACK'i yeniden çalıştır
  (`MANUAL_TESTS.md` #4–#5). Store ekran görüntüleri (`docs/store/screenshots`) yenilenmeli.
- [ ] Windows 10 cihaz varsa: opak görünüm, kapsüller düzgün (`MANUAL_TESTS.md` #7).

**macOS gerilemesi (Mac'te, dal geri çekilince):**
- [ ] `npx tauri build --bundles app` ile derle, aç; görünüm `46c7170` ile aynı olmalı.
  `styles.glass.css` yeniden adlandırması ve seçici değişikliği yüzünden gerekli.

## 6. Commit düzeni

Her biri ayrı commit olacak:

1. stil dosyasının paylaşılır hale gelmesi (macOS görünümü değişmeden),
2. Mica ve yerel katman,
3. vurgu rengi,
4. forced-colors.

Hepsi `feat/macos-support` dalına push edilir. Dal tamamlanınca `main`'e PR açılır; repo `main`
için PR kuralı uyguluyor.
