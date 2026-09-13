#!/usr/bin/env python3
"""
Bengali Training Corpus Fetcher & Cleaner for Lekhani IME.
Fetches high-quality, authentic Bengali text across literature, culture,
science, history, and daily conversational dialogues from public sources.
"""

import json
import re
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

TOPICS = [
    # Geography & Cities
    "বাংলাদেশ",
    "ঢাকা",
    "চট্টগ্রাম",
    "সিলেট",
    "রাজশাহী",
    "খুলনা",
    "বরিশাল",
    "রংপুর",
    "ময়মনসিংহ",
    "সুন্দরবন",
    "কক্সবাজার",
    "পদ্মা সেতু",
    # History & National Heritage
    "ভাষা আন্দোলন",
    "বাংলাদেশের স্বাধীনতা যুদ্ধ",
    "শেখ মুজিবুর রহমান",
    "বাঙালি জাতি",
    "জাতীয় স্মৃতিসৌধ",
    "শহীদ মিনার",
    # Language, Literature & Authors
    "বাংলা ভাষা",
    "বাংলা সাহিত্য",
    "রবীন্দ্রনাথ ঠাকুর",
    "কাজী নজরুল ইসলাম",
    "মাইকেল মধুসূদন দত্ত",
    "শরৎচন্দ্র চট্টোপাধ্যায়",
    "হুমায়ূন আহমেদ",
    "জীবনানন্দ দাশ",
    "গীতাঞ্জলি",
    "একুশে পদক",
    # Culture, Festivals & Music
    "পহেলা বৈশাখ",
    "বাঙালি সংস্কৃতি",
    "বাউল",
    "লালন",
    "নজরুল গীতি",
    "রবীন্দ্রসঙ্গীত",
    "বাংলা চলচ্চিত্র",
    # Science, Technology & Computing
    "বিজ্ঞান",
    "কম্পিউটার",
    "ইন্টারনেট",
    "কৃত্রিম বুদ্ধিমত্তা",
    "মোবাইল ফোন",
    "মহাকাশ",
    "পদার্থবিজ্ঞান",
    "রসায়ন",
    "জীববিজ্ঞান",
    # Sports & Recreation
    "ক্রিকেট",
    "বাংলাদেশ জাতীয় ক্রিকেট দল",
    "ফুটবল",
    "খেলাধুলা",
    "হাডুডু",
    # Food & Health
    "বাঙালি রন্ধনশৈলী",
    "ইলিশ",
    "বিরিয়ানি",
    "মিষ্টি",
    "চা",
    "স্বাস্থ্য",
    "চিকিৎসাবিজ্ঞান",
    "যোগব্যায়াম",
    # Nature & Society
    "নদী",
    "বঙ্গোপসাগর",
    "কৃষি",
    "শিক্ষা",
    "পরিবেশ",
    "আবহাওয়া",
    "ঋতু",
    "গ্রীষ্মকাল",
    "বর্ষাকাল",
    "শীতকাল",
]

CONVERSATIONAL_DATA = """
কেমন আছো তুমি? আমি ভালো আছি, তুমি কেমন আছো?
আজকে তোমার সাথে দেখা করতে আসব।
বাসায় সবাই কেমন আছেন? চাচা এবং খালা ভালো আছেন তো?
দেরি হয়ে যাচ্ছে, চলো তাড়াতাড়ি যাই।
আজকে অফিসে অনেক কাজের চাপ ছিল।
তুমি কি এখন ফ্রি আছো? একটু জরুরি কথা ছিল।
কাল সকালে দেখা হবে আমাদের প্রিয় কফি শপে।
এই বইটা পড়ে আমার খুব ভালো লেগেছে।
শার্টটা পরে তোমার কেমন লাগছে? অনেক সুন্দর লাগছে।
চা খাবে নাকি কফি খাবে? এক কাপ লাল চা খাব।
ভাত খেয়েছো নাকি এখনো খাওনি? মাত্রই ভাত খাচ্ছি।
প্যারা নিও না ভাই, সব ঠিক হয়ে যাবে।
সমস্যা নাই, আমি এটা নিজে দেখে নেব।
অনেক ধন্যবাদ তোমাকে এই উপকারের জন্য।
শুভ সকাল! আজকের দিনটি তোমার চমৎকার কাটুক।
শুভ রাত্রি, ভালো থেকো এবং ভালো ঘুমাও।
আজকে ঢাকার আবহাওয়া খুব সুন্দর এবং বৃষ্টি হচ্ছে।
বৃষ্টির দিনে খিচুড়ি আর ইলিশ মাছ খাওয়ার মজাই আলাদা।
গানটা শুনলে মন একদম ভালো হয়ে যায়।
নতুন ছবিটা দেখতে কেমন হয়েছে? অসাধারণ এক সিনেমা।
আমরা সবাই মিলে একসাথে ঘুরতে যাব।
তোমার ফোন নম্বরটা আমাকে একটু পাঠাও।
ফোন করেছিলাম তোমাকে, কিন্তু তুমি ধরনি।
কাজটা খুব দ্রুত শেষ করতে হবে আমাদের।
পরবর্তী ট্রেন কখন আসবে কিছু জানা আছে?
বিশ্ববিদ্যালয়ে ভর্তি পরীক্ষা খুব সন্নিকটে।
পড়াশোনা কেমন চলছে তোমার? ভালোভাবেই প্রস্তুতি নিচ্ছি।
তুমি কি কালকের ফুটবল ম্যাচটা দেখেছিলে?
বাংলাদেশ ক্রিকেট দল আজ অসাধারণ জয় লাভ করেছে।
তোমার প্রিয় শখ কী? বই পড়া এবং গান শোনা আমার খুব পছন্দ।
সরাসরি কথা বললে সব ভুল বোঝাবুঝি দূর হয়ে যায়।
নিজের স্বাস্থ্যের প্রতি সবসময় যত্ন নেওয়া উচিত।
প্রতিদিন সকালে একটু হাঁটাহাঁটি করা স্বাস্থ্যের জন্য ভালো।
খোদা হাফেজ, আবার দেখা হবে আমাদের।
আল্লাহ হাফেজ, সাবধানে যেও পথে।
ইনশাআল্লাহ আমাদের সব স্বপ্ন পূরণ হবে।
আলহামদুলিল্লাহ আমি এখন আগের চেয়ে অনেক সুস্থ আছি।
মাশাল্লাহ বাড়িটা দেখতে খুব সুন্দর হয়েছে।
দেরি না করে এখনই রওনা হওয়া দরকার।
তুমি কোথায় আছো এখন? আমি বাসার সামনে দাঁড়িয়ে আছি।
তোমার সাথে কথা বলে সত্যিই খুব ভালো লাগল।
কিছু মনে করো না, আমি একটু ব্যস্ত ছিলাম তখন।
মনে থাকবে তোমার এই সুন্দর উপহারটি।
আজকের দিনটা ছিল একদম অন্যরকম আনন্দের।
"""


def fetch_wikipedia_extract(title: str) -> str:
    url = (
        "https://bn.wikipedia.org/w/api.php?"
        + urllib.parse.urlencode(
            {
                "action": "query",
                "format": "json",
                "prop": "extracts",
                "explaintext": "1",
                "titles": title,
            }
        )
    )
    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": "LekhaniTrainer/1.0 (https://github.com/openbangla/lekhani)"
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            pages = data.get("query", {}).get("pages", {})
            for page in pages.values():
                return page.get("extract", "")
    except Exception as e:
        print(f"  [!] Failed to fetch '{title}': {e}", file=sys.stderr)
    return ""


def clean_text(raw: str) -> str:
    # Remove section headings e.g. == পরিচিতি == or === ইতিহাস ===
    text = re.sub(r"={2,}[^=\n]+={2,}", "\n", raw)
    # Remove references like [১], [২], [citation needed]
    text = re.sub(r"\[[^\]]*\]", "", text)
    # Remove URLs
    text = re.sub(r"https?://\S+", "", text)
    # Clean up empty lines
    lines = []
    for line in text.splitlines():
        line = line.strip()
        # Keep lines that have meaningful Bengali content
        bengali_chars = sum(1 for c in line if "\u0980" <= c <= "\u09ff")
        if bengali_chars >= 15:
            lines.append(line)
    return "\n".join(lines)


def main():
    out_dir = Path("data")
    out_dir.mkdir(parents=True, exist_ok=True)
    out_file = out_dir / "bengali_training_corpus.txt"

    print(f"[*] Fetching Bengali corpus across {len(TOPICS)} topics...")
    all_chunks = [CONVERSATIONAL_DATA.strip()]

    total_chars = 0
    for idx, topic in enumerate(TOPICS, 1):
        print(f"[{idx}/{len(TOPICS)}] Fetching: {topic}...", end=" ", flush=True)
        raw = fetch_wikipedia_extract(topic)
        cleaned = clean_text(raw)
        char_count = len(cleaned)
        print(f"({char_count:,} chars)")
        if char_count > 0:
            all_chunks.append(cleaned)
            total_chars += char_count
        time.sleep(0.3)  # Respect API rate limits

    full_corpus = "\n\n".join(all_chunks)
    out_file.write_text(full_corpus, encoding="utf-8")

    word_count = len(full_corpus.split())
    line_count = len(full_corpus.splitlines())

    print("\n" + "=" * 55)
    print("✅ Training Corpus Successfully Created!")
    print(f"📁 Output file:  {out_file.resolve()}")
    print(f"📝 Total Words:  {word_count:,}")
    print(f"🔤 Total Chars:  {len(full_corpus):,}")
    print(f"📄 Total Lines:  {line_count:,}")
    print("=" * 55)
    print("\nTo train your Lekhani personal model right now, run:")
    print(f"  cargo run --bin lekhani -- ai train --input {out_file} --user")
    print(f"  (or if installed: lekhani ai train --input {out_file} --user)")


if __name__ == "__main__":
    main()
