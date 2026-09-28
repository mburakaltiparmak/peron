; Peron NSIS hooks (bundle.windows.nsis.installerHooks). Used by the direct-download (site/GitHub)
; installer; the Microsoft Store build is MSIX and needs none of this.
;
; The language picked in the installer's language selector becomes the app's default language
; until the user changes it in the app (see src-tauri/crates/core/src/i18n.rs, Lang::resolve).

!macro NSIS_HOOK_POSTINSTALL
  ${If} $LANGUAGE == 1055
    WriteRegStr HKCU "Software\Peron" "InstallerLanguage" "tr"
  ${Else}
    WriteRegStr HKCU "Software\Peron" "InstallerLanguage" "en"
  ${EndIf}
  ; Diagnostics log folder (the app also creates it on start; see src-tauri/src/log.rs).
  CreateDirectory "$DOCUMENTS\Peron\Logs"
!macroend

; Clean uninstall. Skipped for updates: Tauri's installer runs the old uninstaller with /UPDATE,
; and wiping these there would silently turn off the user's autostart on every update.
; App data (%APPDATA% and %LOCALAPPDATA% \com.peron.app: settings, feedback limit,
; notification icon, history) is removed by the "Delete application data" checkbox, which starts
; checked (our template fork: src-tauri/nsis/installer.nsi). Silent /S uninstalls keep the data.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    DeleteRegKey HKCU "Software\Peron"
    ; Toast identity registered at runtime by src-tauri/src/notify.rs
    DeleteRegKey HKCU "Software\Classes\AppUserModelId\com.peron.app"
    ; Autostart entry written by tauri-plugin-autostart (value name = productName)
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Peron"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "Peron"
    ; With "Delete application data": our log files in Documents. Folders go only when empty, so
    ; anything the user put there stays.
    ${If} $DeleteAppDataCheckboxState = 1
      Delete "$DOCUMENTS\Peron\Logs\peron.log"
      Delete "$DOCUMENTS\Peron\Logs\peron.old.log"
      RMDir "$DOCUMENTS\Peron\Logs"
      RMDir "$DOCUMENTS\Peron"
    ${EndIf}
  ${EndIf}
!macroend
