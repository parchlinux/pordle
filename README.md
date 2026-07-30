<picture>
  <source media="(prefers-color-scheme: dark)" srcset="data/icons/com.parchlinux.pordle.svg">
  <img alt="پردل" src="data/icons/com.parchlinux.pordle.svg" width="120" align="left" style="margin-right: 24px">
</picture>

<br clear="all">

پردل یک بازی وردل فارسی برای پارچ لینوکس است که با Rust و GTK4 و libadwaita ساخته شده. هر روز یک کلمه پنج حرفی جدید حدس بزنید و ببینید چند حدس لازم دارید.

Pordle is a Persian Wordle game for Parch Linux built with Rust, GTK4, and libadwaita. Guess a new five letter word every day and see how many guesses you need.

<br>

**بازی در دو حالت**

در حالت روزانه همه کاربران یک کلمه یکسان بر اساس تاریخ دریافت می‌کنند و نتیجه هر روز ذخیره می‌شود. در حالت آزاد کلمات تصادفی برای تمرین و تکرار در اختیار شماست. می‌توانید با صفحه کلید فیزیکی یا صفحه کلید لمسی فارسی داخل برنامه تایپ کنید.

**Two game modes**

Daily mode gives everyone the same word based on the date and saves your result for the day. Practice mode serves random words for unlimited play. Type using either your physical keyboard or the on-screen Persian keyboard.

<br>

**رنگ‌ها**

سبز یعنی حرف در جای درست، زرد یعنی حرف در کلمه هست اما جای آن درست نیست، و خاکستری تیره یعنی حرف در کلمه نیست.

**Colors**

Green means the letter is in the right position, yellow means the letter is in the word but in the wrong position, and dark gray means the letter is not in the word.

<br>

**منبع کلمات / Word Database**

بانک اطلاعات کلمات فارسی اولیه این برنامه از پروژه [wordle-farsi](https://github.com/PedramH/wordle-farsi) توسط PedramH استخراج شده است.

The initial Persian dictionary used by Pordle is sourced from [wordle-farsi by PedramH](https://github.com/PedramH/wordle-farsi).

<br>

**مشارکت و افزودن کلمات جدید / Contributing**

اگر می‌خواهید کلمه جدیدی اضافه کنید یا در پروژه مشارکت داشته باشید، لطفاً فایل [CONTRIBUTING.md](CONTRIBUTING.md) را مطالعه کنید.

To add new 5-letter Persian words or contribute to the project, please refer to [CONTRIBUTING.md](CONTRIBUTING.md).

<br>

**ساخت و نصب / Building**

```bash
cargo build --release
./target/release/pordle
```

وابستگی‌ها: GTK4، libadwaita، و Rust. کلمات به صورت پیش‌فرض در خود برنامه کامپایل شده‌اند و امکان اضافه کردن از `data/words.txt` نیز وجود دارد.

Dependencies are GTK4, libadwaita, and Rust. Words are embedded directly inside the application binary on build.

<br>

**مجوز**

این پروژه تحت مجوز AGPL-3.0 منتشر شده است.
