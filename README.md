<div align="center">
  <a href='https://seo-fast.ru/?r=3060386' target='_blank'><img src='https://seo-fast.ru/site_banners/img/banner88x31.gif' width='220' height='51' border='0' alt='seofast-ru' /></a>
</div>

        
**A website that pays for micro tasks like watching YouTube videos or browsing other websites. Please consider helping me by entering my affiliate link [here](https://seo-fast.ru/?r=3060386). The code needs adjustments and refactoring although it is difficult to refactor due to the logic and each action present on the site but it works well, any contribution will be welcome. If you have any questions, you can call me [here](https://t.me/westsidedev)**

<h2>DOWNLOAD:</h2>

+ **Linux** *(requires Ubuntu 24 or GLIBC_2.39)*

  Make sure the browser and driver binaries are located in /bin, the software will look for them in this location. The browser name must only match its name, otherwise you must rename it, the software only supports the brave browser, if you don't have it on your machine you must install it.

  Ex: *brave-browser* for *brave* only

  **Go to the release page, or click [here](https://github.com/westsidedev/seofast-ru/releases)**

+ **Termux** *(arm64)*

  **Go to the release page, or click [here](https://github.com/westsidedev/seofast-ru/releases)**

+ **Windows**

  **Go to the release page, or click [here](https://github.com/westsidedev/seofast-ru/releases)**

<h2>INSTALATION:</h2>

After downloading, create a folder and extract the zip inside it, then give it permission, in termux you must put it in your HOME

+ **Linux**

```bash
mkdir seofast
mv seofast-linux.zip seofast
cd seofast
unzip seofast-linux.zip
chmod +x aviso-bz
./aviso-bz
```

+ **Termux**

```bash
pkg update 
pkg upgrade 
pkg install x11-repo 
pkg install tur-repo 
pkg install chromium
pkg install zip
termux-setup-storage
mv /sdcard/Download/seofast-termux.zip $HOME
mkdir seofast
mv seofast-termux.zip seofast
cd seofast
unzip seofast-termux.zip
chmod +x aviso-bz
./aviso-bz
```
  
<h2>BUILD:</h2>

**If your distribution is different from Ubuntu 24 or lower GLIBC_2.39 consider building from source**

+ **1. Install the latest version of [Rust](https://www.rust-lang.org/tools/install)**
+ **2. Clone this repository**
+ **3. run `cargo build --release`**

<h2>COMMANDS:</h5>

```
--email      Email used for login in seofast
--passw      Password used for login in seofast
--start      Start software after first execution
--headless   Active headless mode (OPTIONAL)
--help       Show this message
--YT         Youtube mode
--SF         Surfing mode
--All        Youtube and surfing mode
```
<h2>EXAMPLE:</h5>

**FIRST EXECUTION:**

```bash
./seofast-ru --email xxxx@xxxx --passw 123456 --YT --headless
```

**START**

```bash
./seofast-ru --start --YT --headless
```

<h2>⚠️DISCLAIMER⚠️</h2>
<h4>This software aims to watch all videos on the site fairly. I am not responsible if your account is banned, the software also only supports 1 account, this does not violate the site's rules. If your account is restricted, talk to support and send them the link to this software explaining what it does, it handles everything fairly and does not involve any cheating</h4>

<h2 align="center"><a href="LICENSE.txt">Apache License</a></h2>
<div align="center"><b>Copyright &copy; 2024 westsidedev. All rights reserved.</b></div>

