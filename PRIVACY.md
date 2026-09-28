# Peron — Gizlilik Politikası / Privacy Policy

**Yürürlük / Effective:** 2026-09-25 · **Sürüm / Applies to:** Peron 0.1.0-beta.1 and later
**Yayın adresi / Published at:** https://burakaltiparmak.dev/products/peron/privacy · https://github.com/mburakaltiparmak/peron/blob/main/PRIVACY.md

---

## Türkçe

Peron, bilgisayarınızda açık olan portları ve bunları kullanan süreçleri gösteren bir masaüstü uygulamasıdır. Geliştirici: M. Burak Altıparmak ("biz").

### Özet
- Peron **hesap istemez, reklam göstermez, telemetri/analitik toplamaz.**
- Port listesi, süreç adları, komut satırları, klasör yolları ve kaynak kullanımı **yalnızca cihazınızda** işlenir; hiçbir yere gönderilmez.
- Cihazdan veri çıkan yalnızca şu durumlar vardır: **(1)** siz geri bildirim formunu gönderdiğinizde, **(2)** Microsoft Store dışından indirilen sürümlerde isteğe bağlı güncelleme kontrolü, **(3)** sunucu eklediyseniz, SSH ile yalnızca kendi sunucunuzla (bkz. 3a).

### 1. Geri bildirim formu (isteğe bağlı)
"İletişim" sekmesinden mesaj gönderdiğinizde şu bilgiler **Web3Forms** (Web3Forms.com) aracılığıyla e-posta olarak geliştiriciye (mburakaltiparmak@gmail.com) iletilir:
- Yazdığınız mesaj ve seçtiğiniz tür (hata / öneri / diğer)
- İsteğe bağlı olarak adınız ve yanıt için e-posta adresiniz
- "Sürüm ve işletim sistemi bilgisini ekle" seçiliyse Peron sürümü, işletim sistemi adı/sürümü ve uygulama dili

**Amaç:** geri bildiriminizi okumak ve isterseniz yanıtlamak. **Hukuki dayanak:** açık rızanız (formu kendiniz gönderirsiniz).
Web3Forms bu veriyi e-postayı iletmek için işleyen üçüncü taraf bir hizmettir; kendi gizlilik politikası geçerlidir: https://web3forms.com/privacy. Web3Forms'a bağlanırken IP adresiniz, her internet isteğinde olduğu gibi, bu hizmet tarafından görülebilir.
Mesajlar geliştiricinin e-posta kutusunda, konusu kapanana kadar ve en fazla **24 ay** saklanır, sonra silinir.

### 2. Güncelleme kontrolü (yalnızca site/GitHub sürümü)
Microsoft Store dışından (burakaltiparmak.dev veya GitHub) indirilen sürüm, günde en fazla bir kez GitHub'ın herkese açık API'sinden (api.github.com) yayımlanan sürüm listesini okur. Bu istekte sizinle ilgili bir bilgi gönderilmez; GitHub isteği yapanın IP adresini görebilir (GitHub gizlilik bildirimi: https://docs.github.com/site-policy/privacy-policies/github-general-privacy-statement). Ayarlar > "Yeni sürüm çıktığında haber ver" seçeneğiyle kapatılabilir. **Microsoft Store sürümü bu kontrolü hiç yapmaz;** güncellemeleri Store üzerinden alır.

### 3. Cihazınızda saklanan veriler
Aşağıdakiler yalnızca bilgisayarınızda, kullanıcı hesabınızın uygulama veri klasöründe tutulur ve kimseyle paylaşılmaz:
- **Ayarlar** (dil, tema, filtreler, hatırlatma süreleri)
- **Geri bildirim gönderim zamanları** (kötüye kullanımı önlemek için saatte/günde gönderim sınırı; en fazla 24 saat)
- **Tanılama günlüğü** (Windows'ta `Belgeler\Peron\Logs`, en fazla ~1 MB; yalnızca teknik hata iletileri, port listesi veya komut satırı içermez). Ayarlar > "Günlük klasörünü aç" ile görebilir, silebilirsiniz. Belgeler klasörünüz OneDrive ile eşitleniyorsa bu dosyalar da OneDrive'ınıza eşitlenir.
- **Port geçmişi** (hangi portun ne zaman açılıp kapandığı, süreç adı, PID, proje klasör adı, adresler; en fazla 30 gün / 2.000 olay). Geçmiş sekmesinden görebilir, dışa aktarabilir veya temizleyebilirsiniz.
- **Sunucu listesi** (uzak izleme için eklediğiniz sunucuların adı ve SSH hedefi). SSH anahtarları veya parolalar Peron tarafından saklanmaz.
- **Dışa aktarılan dosyalar** yalnızca siz istediğinizde, İndirilenler klasörünüze yazılır.

### 3a. Uzak sunucu izleme (isteğe bağlı)
Bir sunucu eklediğinizde Peron, bilgisayarınızdaki `ssh` istemcisini ve sizin SSH anahtarlarınızı kullanarak o sunucuda `peron-cli`'ı çalıştırır. Port ve süreç bilgileri **doğrudan sunucunuzdan bilgisayarınıza** SSH ile şifreli olarak gelir; geliştiriciye veya herhangi bir üçüncü tarafa gitmez. Sunucuda yeni bir servis çalıştırılmaz ya da port açılmaz.

Kaldırma: Microsoft Store sürümünü kaldırdığınızda bu veriler Windows tarafından otomatik silinir; yalnızca `Belgeler\Peron\Logs` klasörü kalır, elle silebilirsiniz. Site/GitHub sürümünde kaldırıcıdaki "Uygulama verilerini sil" seçeneği varsayılan olarak işaretlidir ve günlük dosyalarını da siler; verileri tutmak isterseniz işareti kaldırın.

### 4. Süreç sonlandırma
Peron bir süreci yalnızca siz onay penceresinde açıkça onayladığınızda sonlandırır. Bu işlem tamamen yereldir.

### 5. Çocuklar
Peron geliştiricilere yönelik bir araçtır; bilerek 13 yaşından küçüklerden veri toplamayız.

### 6. Haklarınız ve iletişim
Gönderdiğiniz geri bildirimin kopyasını, düzeltilmesini veya silinmesini istemek için: **mburakaltiparmak@gmail.com** (konu: "Peron gizlilik"). 30 gün içinde yanıt veririz. KVKK ve GDPR kapsamındaki haklarınız saklıdır.

### 7. Değişiklikler
Bu politika değişirse yeni sürümü aynı adreste, yürürlük tarihini güncelleyerek yayımlarız. Önemli değişiklikler sürüm notlarında belirtilir.

---

## English

Peron is a desktop app that shows the ports open on your computer and the processes using them. Developer: M. Burak Altıparmak ("we").

### Summary
- Peron has **no accounts, no ads, no telemetry or analytics.**
- The port list, process names, command lines, folder paths and resource usage are processed **only on your device** and are never sent anywhere.
- Data leaves your device only in these cases: **(1)** when you submit the feedback form, **(2)** the optional update check in builds downloaded outside the Microsoft Store, **(3)** if you add a server, over SSH with that server only (see 3a).

### 1. Feedback form (optional)
When you send a message from the "Contact" tab, the following is delivered by email to the developer (mburakaltiparmak@gmail.com) via **Web3Forms** (web3forms.com):
- Your message and the type you chose (bug / suggestion / other)
- Optionally your name and a reply email address
- If "Include app version and operating system" is checked: Peron version, OS name/version and app language

**Purpose:** to read and, if you wish, answer your feedback. **Legal basis:** your consent (you submit the form yourself).
Web3Forms is a third-party service that processes this data to deliver the email; its privacy policy applies: https://web3forms.com/privacy. As with any internet request, Web3Forms can see your IP address when you connect.
Messages are kept in the developer's mailbox until the matter is closed and for at most **24 months**, then deleted.

### 2. Update check (site/GitHub builds only)
The version downloaded outside the Microsoft Store (burakaltiparmak.dev or GitHub) reads the list of published releases from GitHub's public API (api.github.com) at most once a day. No information about you is sent; GitHub can see the requesting IP address (GitHub privacy statement: https://docs.github.com/site-policy/privacy-policies/github-general-privacy-statement). Turn it off in Settings → "Tell me when a new version is out". **The Microsoft Store version never performs this check;** it is updated through the Store.

### 3. Data stored on your device
Kept only on your computer, in your user account's app data folder, and never shared:
- **Settings** (language, theme, filters, reminder durations)
- **Feedback send times** (hourly/daily send limit against abuse; at most 24 hours)
- **Diagnostics log** (on Windows `Documents\Peron\Logs`, max ~1 MB; technical error messages only, no port lists or command lines). View or delete it via Settings → "Open log folder". If your Documents folder is synced by OneDrive, these files are synced to your OneDrive too.
- **Port history** (which port opened/closed when, process name, PID, project folder name, addresses; at most 30 days / 2,000 events). View, export or clear it in the History tab.
- **Server list** (name and SSH target of servers you add for the remote view). Peron never stores SSH keys or passwords.
- **Exported files** are written only when you ask, to your Downloads folder.

### 3a. Remote server view (optional)
When you add a server, Peron runs `peron-cli` on it using the `ssh` client on your computer and your own SSH keys. Port and process data travel **directly from your server to your computer**, encrypted by SSH; they never go to the developer or any third party. No service is installed and no port is opened on the server.

Uninstalling: the Microsoft Store version's data is removed by Windows automatically, except the `Documents\Peron\Logs` folder, which you can delete yourself. For the site/GitHub version, "Delete application data" in the uninstaller is checked by default and also removes the log files; untick it to keep your data.

### 4. Ending processes
Peron ends a process only after you explicitly confirm it in the confirmation dialog. This happens entirely on your device.

### 5. Children
Peron is a tool for developers; we do not knowingly collect data from children under 13.

### 6. Your rights and contact
To request a copy, correction or deletion of feedback you sent: **mburakaltiparmak@gmail.com** (subject: "Peron privacy"). We reply within 30 days. Your rights under GDPR and Turkish KVKK are unaffected.

### 7. Changes
If this policy changes we publish the new version at the same address with an updated effective date. Material changes are mentioned in the release notes.
