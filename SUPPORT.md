# Peron — Destek / Support

**Web:** https://burakaltiparmak.dev/contact · **E-posta / Email:** mburakaltiparmak@gmail.com
**Uygulama içi / In-app:** İletişim sekmesi / Contact tab

## Türkçe

**Sorun bildirirken** İletişim sekmesindeki formu kullanın; "sürüm ve işletim sistemi bilgisini ekle" seçeneği açık kalsın. Gerekirse Ayarlar > Tanılama > "Günlük klasörünü aç" ile `peron.log` dosyasını e-postaya ekleyin (kişisel veri içermez).

**Sık sorulanlar**
- *Bazı portların sahibi "—" veya "?" görünüyor.* Başka kullanıcıya ya da yönetici haklarıyla çalışan sürece ait. Windows (site/GitHub sürümü): araç çubuğundaki **Yönetici** düğmesi. Linux: Peron'u root olarak çalıştırın. Microsoft Store sürümü bu süreçleri listeler ama ayrıntılarını gösteremez.
- *"Kapat" düğmesi pasif.* Süreç korumalı bir sistem sürecidir (ör. `svchost.exe`, `System`); Peron bunları bilerek kapatmaz.
- *Windows "bilgisayarınız korundu" diyor.* Site/GitHub kurulum dosyası henüz kod imzalı değil: **Ek bilgi → Yine de çalıştır**. İmzalı sürüm için Microsoft Store'u kullanın.
- *Bildirim gelmiyor.* Windows Ayarlar > Bildirimler'de Peron'a izin verildiğini ve Odak/Rahatsız Etmeyin kapalı olduğunu kontrol edin. Ayarlar > Hatırlatmalar süresini düşürerek deneyin.
- *Windows ile başlamıyor.* Ayarlar > Başlangıç'ı açın. Store sürümünde Windows Ayarlar > Uygulamalar > Başlangıç'ta Peron'un açık olduğundan emin olun.
- *Dil yanlış.* Ayarlar > Genel > Dil veya tepsi menüsü > Dil.
- *Kaldırma.* Store: Windows Ayarlar > Uygulamalar. Site/GitHub: Uygulamalar'dan kaldırın; "Uygulama verilerini sil" varsayılan olarak işaretlidir (ayarları tutmak için işareti kaldırın).

## English

**When reporting a problem** use the form in the Contact tab and keep "include app version and operating system" checked. If needed attach `peron.log` from Settings → Diagnostics → "Open log folder" (no personal data).

**FAQ**
- *Some ports show "—" or "?" as owner.* The process belongs to another user or runs elevated. Windows (site/GitHub build): the **Admin** button in the toolbar. Linux: run Peron as root. The Microsoft Store build lists such processes but can't show their details.
- *The "Close" button is disabled.* It's a protected system process (e.g. `svchost.exe`, `System`); Peron deliberately won't end it.
- *Windows says "Windows protected your PC".* The site/GitHub installer isn't code-signed yet: **More info → Run anyway**. Use the Microsoft Store for a signed build.
- *No reminders appear.* Allow Peron in Windows Settings → Notifications and make sure Focus/Do not disturb is off. Try a lower threshold in Settings → Reminders.
- *Doesn't start with Windows.* Turn on Settings → Startup. For the Store build also check Windows Settings → Apps → Startup.
- *Wrong language.* Settings → General → Language, or tray menu → Language.
- *Uninstall.* Store: Windows Settings → Apps. Site/GitHub: uninstall from Apps; "Delete application data" is checked by default (untick it to keep your settings).
