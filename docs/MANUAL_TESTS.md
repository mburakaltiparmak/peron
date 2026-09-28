# Peron — Manuel Test Rehberi

Otomatik testlerin (Rust, Vitest, sessiz kurulum, uzak uçtan uca) kapsamadığı, elle yapılan testler.
Her testin adımları, beklenen sonucu ve son durumu burada tutulur. Yayın öncesi kontrol listesi:
[`docs/store/RELEASE_CHECKLIST.md`](store/RELEASE_CHECKLIST.md).

**Durum:** ✅ geçti · ⏳ yapılmadı · ❌ sorun var (not düşülür)

| # | Test | Durum | Son tarih / not |
|---|---|---|---|
| 1 | NSIS etkileşimli kurulum ve kaldırma | ✅ | 2026-09-26 |
| 2 | Ekran Okuyucu (Narrator) | ✅ | 2026-09-26 |
| 3 | Yüksek karşıtlık | ✅ | 2026-09-26 (CDP öykünmesiyle; göz kontrolü önerilir) |
| 4 | MSIX kurulum ve davranış | ✅ | 2026-09-26 (yer tutucu kimlik; günlük klasörü düzeltmesi dahil) |
| 5 | WACK (Store sertifika testi) | ✅ | 2026-09-26: OVERALL PASS (24 test); yalnızca isteğe bağlı "Engellenen yürütülebilir dosyalar" FAIL — beklenen, aşağıya bkz. |
| 6 | MSIX temizliği | ✅ | 2026-09-26 |
| 7 | Windows 10 | ⏳ | 2026-09-26: şu an cihaz yok; ertelendi |
| 8 | Store ekran görüntüleri | ✅ | 2026-09-26: TR+EN, 6'şar adet, 1920×1080, Store arayüzü |
| 9 | Gerçek kimlikli MSIX | ✅ | 2026-09-27: kurulum, davranış, WACK ve kaldırma |

Arayüz üzerinden ayrıca doğrulanan akışlar (2026-09-26): dışa aktarma, uzak sunucu, hızlı sunucu
değiştirme, geçmiş, bildirim, yerel kapatma, tepsi, Web3Forms.

---

## Hazırlık

- **Sınama Modu (Test Mode) kapalı olmalı.** Hiçbir test bunu gerektirmez; açıksa yönetici PowerShell'de
  `bcdedit /set testsigning off` ve yeniden başlatma.
- Güncel derlemeler:
  ```powershell
  npm run tauri build                                  # NSIS: src-tauri\target\release\bundle\nsis\Peron_<sürüm>_x64-setup.exe
  .\packaging\msix\build-msix.ps1 -SkipBuild -SelfSign # MSIX: src-tauri\target\msix\Peron_<sürüm>_x64.msix
  ```
- NSIS ve MSIX sürümleri aynı anda kurulu olmasın (tepside iki Peron olur, sonuçlar karışır).
- Test portu açmak için: `python -m http.server 8765` (yerel), `python -m http.server 8799 --bind 0.0.0.0` (ağa açık).

---

## 1. NSIS etkileşimli kurulum ve kaldırma

Kurulum dosyası: `src-tauri\target\release\bundle\nsis\Peron_0.1.0-beta.1_x64-setup.exe`

**Kurulum**
1. SmartScreen uyarısı çıkabilir (kurulum imzasız): **Ek bilgi → Yine de çalıştır**.
2. Dil seçimi sorulmalı (Türkçe / English); ekranlar seçilen dilde olmalı. UAC (yönetici izni) **istenmemeli**.
3. İlk açılış kurulumda seçilen dilde olmalı (kayıtlı ayar varsa o önceliklidir).
4. Başlat menüsünde "Peron" kısayolu ve logosu.
5. Windows Ayarlar > Uygulamalar > Yüklü uygulamalar: Peron, sürüm 0.1.0-beta.1, yayıncı adı.
6. `Belgeler\Peron\Logs` klasörü oluşmuş olmalı.
7. Pencere kapatılınca tepsiye inmeli; tepsi simgesine **sağ tık** → menü (dil, portlar, Çıkış).
8. Ayarlar > Başlangıç açılırsa: oturum açılışında pencere açılmadan tepside başlamalı.
9. Tema: Açık/Koyu seçince **anında** önizleme, başlık çubuğu dahil; İptal eski temaya döner.

**Kaldırma** (önce tepsiden **Çıkış**; kurulum klasörü Dosya Gezgini'nde açık olmasın)
1. Yüklü uygulamalar > Peron > Kaldır.
2. **"Uygulama verilerini sil"** varsayılan olarak işaretli olmalı.
3. Sonra kalmamalı: `%LOCALAPPDATA%\Peron`, `%APPDATA%\com.peron.app`, `%LOCALAPPDATA%\com.peron.app`,
   `Belgeler\Peron\Logs` içindeki günlükler (klasör boşsa o da), Başlat kısayolu, Yüklü uygulamalar kaydı,
   Görev Yöneticisi > Başlangıç uygulamaları'ndaki Peron.

Otomatik karşılığı: `.\packaging\test-nsis-silent.ps1` (sessiz kurulum/kaldırma, 16 kontrol).

---

## 2. Ekran Okuyucu (Narrator)

| Tuş | İşlev |
|---|---|
| `Ctrl + Win + Enter` | Narrator aç/kapat |
| `Ctrl` | Konuşmayı sustur |
| `Tab` / `Shift+Tab` | Sonraki/önceki öğe |
| `Caps Lock + ←/→` | Öğeleri tek tek tarama |
| `Ctrl + Alt + ←↑→↓` | Tabloda hücre hücre gezme |
| `Caps Lock + Tab` | Odaktaki öğeyi yeniden okut |

Uygulama kısayolları: `Ctrl+F` ara · `F5` yenile · `Delete` seçili portu kapat · `Esc` kapat.

| # | Adım | Beklenen |
|---|---|---|
| 1 | Üst çubukta `Tab` ile gezin | Sekmeler "Portlar, sekme, seçili"; düğmeler anlamlı adlarla ("Yönetici", "Dışa aktar", "Yenile", "Ayarlar"). Adsız "düğme" yok. |
| 2 | `Ctrl+F` | "Ara, arama kutusu" |
| 3 | Makine seçici | "Makine, açılan kutu, Bu bilgisayar" |
| 4 | Filtre çipleri | "Tümü 13, radyo düğmesi, seçili"; oklarla seçim değişimi okunur |
| 5 | Tabloya girip oklarla gezin | Satır içeriği ve "seçili"; `Ctrl+Alt+→` ile sütunlar |
| 6 | Satır seçiliyken `Delete` | Onay penceresinin başlığı ve açıklaması okunur; odak pencerede kalır |
| 7 | `Esc` | Pencere kapanır, odak tabloya döner |
| 8 | `Delete` → "Süreci sonlandır" | Sonuç bildirimi kendiliğinden okunur |
| 9 | Dışa aktar → TXT | Menü öğeleri ve "Kaydedildi: …" okunur |
| 10 | Ayarlar'da `Tab` | Her alan etiketiyle; tema düğmelerinde "basılı" durumu |
| 11 | Geçmiş sekmesi | Olay sayısı, "Ara", tablo |
| 12 | İletişim → kısa mesajla Gönder | "Mesaj çok kısa…" kendiliğinden okunur (gerçek gönderim yapmayın) |
| 13 | Tepsi (`Win+B`, oklar) | "Peron — N açık port"; `Enter` pencereyi, `Shift+F10` menüyü açar |

## 3. Yüksek karşıtlık

`Sol Alt + Sol Shift + Print Screen` (veya Ayarlar > Erişilebilirlik > Karşıtlık temaları).
Seçili sekme, görünüm (Dinleyen/Tüm), filtre çipi ve tablo satırı sistem vurgu rengiyle belirgin olmalı;
metin ve kenarlıklar okunmalı; onay pencereleri çerçeveli görünmeli.

---

## 4. MSIX kurulum ve davranış

**Kurulum** — yönetici PowerShell, **bu sırayla**:
```powershell
Import-Certificate -FilePath "$env:TEMP\peron-msix-real.cer" -CertStoreLocation Cert:\LocalMachine\TrustedPeople
Add-AppxPackage "C:\Users\phant\Desktop\Burak\Projeler\peron\src-tauri\target\msix\Peron_0.1.0-beta.1_x64.msix"
```
- `peron-msix-real.cer` yoksa: `Export-Certificate -Cert Cert:\CurrentUser\My\D3564EA6954967088AFA902D5E690D9E07AF1319 -FilePath "$env:TEMP\peron-msix-real.cer"` (test sertifikası `-SelfSign` ile üretilir; parmak izi değişirse betik çıktısındakini kullanın)
- Aynı sürüm zaten kuruluysa önce `Get-AppxPackage M.BurakAltiparmak.Peron | Remove-AppxPackage`.
- `0x800B0109` hatası = sertifika henüz güvenilir değil (komut sırası).

| # | Adım | Beklenen |
|---|---|---|
| 1 | Başlat'tan "Peron" | Logosuyla açılır; dil Windows dilinden |
| 2 | Araç çubuğu | **"Yönetici" düğmesi yok** |
| 3 | Ayarlar | "Yeni sürüm çıktığında haber ver" **yok** |
| 4 | Ayarlar > "Windows ile başlat" | Windows Ayarlar > Uygulamalar > Başlangıç'ta Peron **açık** |
| 5 | Oturumu kapat/aç | Pencere açılmadan tepside başlar |
| 6 | "Şu süreden uzun açık kalınca" = **1** (tekrar aralığı en az 5), 8765 portunu açıp 1–2 dk bekleyin | Bildirim "Peron" adı ve logosuyla; tıklayınca pencere açılır |
| 7 | 8799'u 0.0.0.0'da açın | "Ağa açık yeni port" bildirimi |
| 8 | Bir portu arayüzden kapatın | Onay penceresi, port kapanır |
| 9 | Tema, tepsi sağ tık, Dışa aktar | NSIS sürümüyle aynı; dosya İndirilenler'de |
| 10 | Ayarlar > "Günlük klasörünü aç" | `Belgeler\Peron\Logs` açılır, içinde `peron.log` |

## 5. WACK (Windows App Certification Kit)

Yönetici PowerShell, 5–10 dk; ekranda pencereler açılıp kapanabilir (bu sırada bilgisayarı kullanmayın).
MSIX kurulu olmasa da çalışır; paketi kendisi kurar/kaldırır.
```powershell
& "${env:ProgramFiles(x86)}\Windows Kits\10\App Certification Kit\appcert.exe" test `
  -appxpackagepath "C:\Users\phant\Desktop\Burak\Projeler\peron\src-tauri\target\msix\Peron_0.1.0-beta.1_x64.msix" `
  -reportoutputpath "$env:TEMP\peron-wack.xml"
```
Rapor: `%TEMP%\peron-wack.xml` (`OVERALL_RESULT="PASS"` beklenir). Yer tutucu kimlik yüzünden
yayıncı/kimlikle ilgili bir uyarı çıkabilir; gerçek kimlikle (test 9) tekrarlanır.
`peron-wack.xml` zaten varsa önce silin (appcert üzerine yazmayı reddedebilir).

**Beklenen sonuç (2026-09-26):** OVERALL_RESULT = PASS. İsteğe bağlı (`OPTIONAL="TRUE"`) **Engellenen yürütülebilir dosyalar** testi FAIL verir ve Store onayını etkilemez: `CreateProcessW`/`ShellExecute(Ex)W` bilerek kullanılıyor (`ssh`, bağlantı/klasör/mail açma, yönetici olarak yeniden başlatma); `cmd.exe` metni Rust standart kütüphanesinden (`std::process` .bat desteği), `bash` yalnızca testlerde. Peron cmd/bash çalıştırmaz. Yeni bir API/dosya adı listelenirse incelenmeli.

## 6. MSIX temizliği

Yönetici PowerShell:
```powershell
Get-AppxPackage M.BurakAltiparmak.Peron | Remove-AppxPackage
Remove-Item Cert:\LocalMachine\TrustedPeople\D3564EA6954967088AFA902D5E690D9E07AF1319
```
Sonra Başlangıç listesinde ve Başlat menüsünde Peron kalmamalı. `Belgeler\Peron\Logs` Store kaldırmasında
**kalır** (Windows Belgeler'e dokunmaz; PRIVACY.md'de yazılı) — elle silinebilir.

---

## 7. Windows 10 (imkân varsa)

Windows 10 1809+ bir makinede (veya sanal makinede) 1. ve 4. testlerin kısa bir turu: kurulum, açılış,
port listesi, kapatma, bildirim, tepsi. WebView2 yüklü değilse NSIS kurulumu indirip kurmalı
(`downloadBootstrapper`).

## 8. Store ekran görüntüleri

Son arayüzle, **MSIX sürümü kurulu hâlde** (Store'daki gibi "Yönetici" düğmesi olmadan), TR ve EN,
1920×1080 (en az 1366×768). Kaydedilecek yer ve adlandırma: `docs/store/screenshots/` (mevcut dosyaların
yerine). Önerilen kareler: Portlar (koyu ve açık tema), detay paneli, kapatma onayı, Geçmiş, uzak sunucu
görünümü, İletişim. Kişisel bilgi (kullanıcı adı, gerçek proje/sunucu adları) görünmemeli.

## 9. Gerçek kimlikli MSIX

Kimlik (`docs/store/PARTNER_CENTER.md`) betiğin varsayılanıdır; GitHub değişkenleri de ayarlı.
```powershell
.\packaging\msix\build-msix.ps1 -SkipBuild            # Store'a yüklenecek imzasız paket → kopyala: target\msix\store\
.\packaging\msix\build-msix.ps1 -SkipBuild -SelfSign  # yerel test paketi (target\msix\)
```
Sonra 4. testteki kurulum komutları (`peron-msix-real.cer`), 4. kontrol listesi, 5 (WACK) ve 6 (temizlik,
paket adı `M.BurakAltiparmak.Peron`). Test bitince mutlaka kaldırın: aynı kimlikle yerel kurulum, Store'dan
kurulumu engeller.
