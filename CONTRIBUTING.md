# راهنمای مشارکت در پردل / Contributing to Pordle

از علاقه شما به مشارکت در پروژه **پردل** سپاسگزاریم! شما می‌توانید با افزودن کلمات جدید به بانک اطلاعاتی یا بهبود کد برنامه، به ما کمک کنید.

Thank you for considering contributing to **Pordle**! You can contribute by adding new Persian words to the word database or improving the code.

---

## 📝 افزودن کلمه جدید / Adding a New Word

کلمات در فایل [`data/words.txt`](data/words.txt) نگهداری می‌شوند و هنگام ساخت برنامه (`cargo build`) به صورت مستقیم درون برنامه جاگذاری و در دیتابیس محلی ذخیره می‌شوند.

Words are stored in [`data/words.txt`](data/words.txt) and embedded directly into the application binary upon compilation.

### شرایط کلمات جدید / Word Requirements:
1. **طول کلمه**: باید دقیقاً **۵ حرف** باشد. (Must be exactly 5 Persian letters).
2. **حروف مجاز**: فقط شامل حروف استاندارد فارسی (`آ-ی`). (Standard Persian alphabet only).
3. **عدم تکرار**: کلمه نباید قبلاً در لیست موجود باشد. (No duplicate words).

---

## 🛠 روش‌های افزودن کلمه / How to Add Words

### روش اول: ویرایش فایل `data/words.txt` (پیشنهادی برای ارسال PR)
1. فایل [`data/words.txt`](data/words.txt) را باز کنید.
2. کلمه ۵ حرفی جدید خود را به ترتیب الفبا اضافه کنید.
3. برنامه را تست کنید:
   ```bash
   cargo test
   cargo run
   ```
4. تغییرات را commit کرده و یک Pull Request ارسال کنید.

### روش دوم: استفاده از ابزار خط فرمان (CLI)
برای اضافه کردن یک کلمه به دیتابیس محلی خود:
```bash
pordle --add-word "آزادی"
```

برای وارد کردن دسته‌ای کلمات از یک فایل متنی:
```bash
pordle --import my_words.txt
```

---

## 🤝 ارسال Pull Request

1. مخزن را **Fork** کنید.
2. یک شاخه (Branch) جدید برای تغییرات خود ایجاد کنید: `git checkout -b feat/add-new-words`.
3. تغییرات خود را اضافه و commit کنید.
4. مطمئن شوید برنامه بدون خطا کامپایل می‌شود و تست‌ها پاس می‌شوند:
   ```bash
   cargo check
   cargo test
   ```
5. شاخه خود را Push کرده و یک **Pull Request** ارسال کنید.

---

## 📚 منبع اولیه کلمات / Word Attribution
بانک اطلاعات کلمات اولیه پردل از پروژه [wordle-farsi](https://github.com/PedramH/wordle-farsi) تهیه شده است.
