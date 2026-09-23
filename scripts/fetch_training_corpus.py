#!/usr/bin/env python3
"""
Bengali Training Corpus Fetcher, Cleaner & Ingestion Pipeline for Lekhani IME.

Features:
- Unicode NFC normalization and zero-width joiner (ZWJ/ZWNJ) sanitization.
- Multi-domain topic fetcher via Bengali Wikipedia REST/Action API.
- Local directory merging (--merge-dir) to ingest offline text archives.
- Streaming XML/BZ2 Wikipedia dump extractor (--dump-bz2).
- Clean sentence segmentation and punctuation formatting.
"""

import argparse
import json
import os
import re
import sys
import time
import unicodedata
import urllib.parse
import urllib.request
from pathlib import Path

# Expanded comprehensive topics across 15+ domains
TOPICS = [
    # 1. Geography, Administrative Divisions & Major Landmarks
    "বাংলাদেশ", "ঢাকা", "চট্টগ্রাম", "সিলেট", "রাজশাহী", "খুলনা", "বরিশাল",
    "রংপুর", "ময়মনসিংহ", "কুমিল্লা", "গাজীপুর", "নারায়ণগঞ্জ", "কক্সবাজার",
    "সুন্দরবন", "পদ্মা সেতু", "বঙ্গবন্ধু যমুনা সেতু", "কুয়াকাটা", "সেন্ট মার্টিন্স দ্বীপ",
    "বান্দরবান", "রাঙ্গামাটি", "খাগড়াছড়ি", "শ্রীমঙ্গল", "জাফলং", "নীলগিরি",
    "পদ্মা নদী", "মেঘনা নদী", "যমুনা নদী", "ব্রহ্মপুত্র নদ", "কর্ণফুলী নদী",
    "বঙ্গোপসাগর", "হিমালয় পর্বতমালা",

    # 2. National History, Liberation & Heritage
    "ভাষা আন্দোলন", "বাংলাদেশের স্বাধীনতা যুদ্ধ", "শেখ মুজিবুর রহমান",
    "বাঙালি জাতি", "জাতীয় স্মৃতিসৌধ", "শহীদ মিনার", "মুজিবনগর সরকার",
    "সাতজন বীরশ্রেষ্ঠ", "মুক্তিবাহিনী", "ছয় দফা আন্দোলন", "ঐতিহাসিক ৭ই মার্চের ভাষণ",
    "অপারেশন সার্চলাইট", "স্মৃতিসৌধ", "জাতীয় সংসদ ভবন", "লালবাগ কেল্লা",
    "সোমপুর মহাবিহার", "মহাস্থানগড়", "ষাট গম্বুজ মসজিদ", "আহসান মঞ্জিল",

    # 3. Bengali Language, Linguistics & Phonetics
    "বাংলা ভাষা", "বাংলা সাহিত্য", "বাংলা বর্ণমালা", "বাংলা ব্যাকরণ",
    "যুক্তবর্ণ", "সাধু ভাষা", "চলিত ভাষা", "আন্তর্জাতিক মাতৃভাষা দিবস",
    "বাংলা একাডেমি", "একুশে পদক", "স্বাধীনতা পুরস্কার",

    # 4. Classic & Modern Literature, Poets & Authors
    "রবীন্দ্রনাথ ঠাকুর", "কাজী নজরুল ইসলাম", "মাইকেল মধুসূদন দত্ত",
    "শরৎচন্দ্র চট্টোপাধ্যায়", "বঙ্কিমচন্দ্র চট্টোপাধ্যায়", "হুমায়ূন আহমেদ",
    "জীবনানন্দ দাশ", "জসীমউদ্দীন", "শামসুর রাহমান", "সুকান্ত ভট্টাচার্য",
    "রোকেয়া সাখাওয়াত হোসেন", "তারাশঙ্কর বন্দ্যোপাধ্যায়", "মানিক বন্দ্যোপাধ্যায়",
    "সৈয়দ মুজতবা আলী", "সৈয়দ শামসুল হক", "আখতারুজ্জামান ইলিয়াস", "আল মাহমুদ",
    "গীতাঞ্জলি", "অগ্নিবীণা", "মেঘনাদবধ কাব্য", "পথের পাঁচালী",

    # 5. Culture, Folklore, Festivals & Music
    "পহেলা বৈশাখ", "বাঙালি সংস্কৃতি", "বাউল", "লালন", "নজরুল গীতি",
    "রবীন্দ্রসঙ্গীত", "ভাটিয়ালি", "ভাওয়াইয়া", "লোকসংগীত", "নবান্ন",
    "পৌষ সংক্রান্তি", "নৌকাবাইচ", "বাংলা চলচ্চিত্র", "সত্যজিৎ রায়",
    "জহির রায়হান", "তারেক মাসুদ", "মৃণাল সেন", "ঢাকাই জামদানি", "নকশী কাঁথা",

    # 6. Science, Computing & Information Technology
    "বিজ্ঞান", "কম্পিউটার", "ইন্টারনেট", "কৃত্রিম বুদ্ধিমত্তা", "যন্ত্রীয় শিখন",
    "মোবাইল ফোন", "সফটওয়্যার", "প্রোগ্রামিং ভাষা", "অ্যালগরিদম", "মহাকাশ",
    "পদার্থবিজ্ঞান", "রসায়ন", "জীববিজ্ঞান", "গণিত", "জ্যোতির্বিজ্ঞান",
    "রোবট", "সাইবার নিরাপত্তা", "ডেটাবেস", "বায়োইনফরমেটিক্স",

    # 7. Law, Governance & Constitutional Framework
    "বাংলাদেশের সংবিধান", "বাংলাদেশ সুপ্রিম কোর্ট", "মৌলিক অধিকার",
    "আইনশাসন", "জাতীয় সংসদ", "স্থানীয় সরকার", "মানবাধিকার", "গণতন্ত্র",
    "বিচার বিভাগ", "নাগরিক অধিকার",

    # 8. Economics, Commerce & Industries
    "বাংলাদেশের অর্থনীতি", "বাংলাদেশ ব্যাংক", "পোশাক শিল্প", "কৃষি",
    "রপ্তানি", "আমদানি", "মুদ্রা", "শেয়ার বাজার", "ক্ষুদ্রঋণ", "রেমিট্যান্স",
    "পাট", "চা শিল্প", "মৎস্য সম্পদ",

    # 9. Health, Medicine & Nutrition
    "স্বাস্থ্য", "চিকিৎসাবিজ্ঞান", "জনস্বাস্থ্য", "পুষ্টি", "রোগ প্রতিরোধ",
    "যোগব্যায়াম", "মানসিক স্বাস্থ্য", "টিকা", "প্রাথমিক চিকিৎসা",

    # 10. Sports & Games
    "ক্রিকেট", "বাংলাদেশ জাতীয় ক্রিকেট দল", "ফুটবল", "খেলাধুলা",
    "হাডুডু", "দাবা", "কাবাডি", "বিশ্বকাপ ক্রিকেট", "ফিফা বিশ্বকাপ",

    # 11. Food, Delicacies & Culinary Arts
    "বাঙালি রন্ধনশৈলী", "ইলিশ", "বিরিয়ানি", "খিচুড়ি", "মিষ্টি",
    "রসগোল্লা", "চমচম", "সন্দেশ", "চা", "কফি", "পিঠা",

    # 12. Nature, Environment, Weather & Climate
    "পরিবেশ", "আবহাওয়া", "জলবায়ু পরিবর্তন", "ঋতু", "গ্রীষ্মকাল",
    "বর্ষাকাল", "শরৎকাল", "হেমন্তকাল", "শীতকাল", "বসন্তকাল",
    "ঘূর্ণিঝড়", "বন্যা", "বন", "জীববৈচিত্র্য", "রয়্যাল বেঙ্গল টাইগার",

    # 13. Education & Academic Institutions
    "শিক্ষা", "ঢাকা বিশ্ববিদ্যালয়", "বাংলাদেশ প্রকৌশল বিশ্ববিদ্যালয়",
    "জাহাঙ্গীরনগর বিশ্ববিদ্যালয়", "রাজশাহী বিশ্ববিদ্যালয়",
    "চট্টগ্রাম বিশ্ববিদ্যালয়", "উচ্চশিক্ষা", "প্রাথমিক শিক্ষা",
]

# Rich everyday conversational dialogues, colloquial idioms, and disambiguation sentences
CONVERSATIONAL_DATA = """
কেমন আছো তুমি? আমি ভালো আছি, তুমি কেমন আছো?
আজকে তোমার সাথে দেখা করতে আসব।
বাসায় সবাই কেমন আছেন? চাচা এবং খালা ভালো আছেন তো?
দেরি হয়ে যাচ্ছে, চলো তাড়াতাড়ি যাই।
আজকে অফিসে অনেক কাজের চাপ ছিল।
তুমি কি এখন ফ্রি আছো? একটু জরুরি কথা ছিল।
কাল সকালে দেখা হবে আমাদের প্রিয় কফি শপে।
এই নতুন বইটা পড়তে আমার খুব ভালো লেগেছে। বই পড়া আমার প্রিয় শখ।
শার্টটা পরে তোমার কেমন লাগছে? অনেক চমৎকার লাগছে। নতুন জামা পরা সুন্দর।
চা খাবে নাকি কফি খাবে? এক কাপ গরম লাল চা খাব।
ভাত খেয়েছো নাকি এখনো খাওনি? মাত্রই ভাত খাচ্ছি।
প্যারা নিও না ভাই, সব ঠিক হয়ে যাবে।
সমস্যা নাই, আমি এটা নিজে গিয়ে দেখে নেব।
অনেক অনেক ধন্যবাদ তোমাকে এই উপকারের জন্য।
শুভ সকাল! আজকের দিনটি তোমার চমৎকার কাটুক।
শুভ রাত্রি, ভালো থেকো এবং ভালো ঘুমাও।
আজকে ঢাকার আবহাওয়া খুব সুন্দর এবং গুঁড়ি গুঁড়ি বৃষ্টি হচ্ছে।
বৃষ্টির দিনে ভুনা খিচুড়ি আর ভাজা ইলিশ মাছ খাওয়ার মজাই আলাদা।
গানটা শুনলে মন একদম ভালো এবং শান্ত হয়ে যায়।
নতুন সিনেমাটা দেখতে কেমন হয়েছে? অসাধারণ এক চলচ্চিত্র।
আমরা সবাই মিলে একসাথে সেন্ট মার্টিন ঘুরতে যাব।
তোমার মোবাইল ফোন নম্বরটা আমাকে একটু পাঠাও।
ফোন করেছিলাম তোমাকে, কিন্তু তুমি কল ধরনি।
কাজটা খুব দ্রুত এবং নিখুঁতভাবে শেষ করতে হবে আমাদের।
পরবর্তী ট্রেন কখন প্ল্যাটফর্মে আসবে কিছু জানা আছে?
বিশ্ববিদ্যালয়ে ভর্তি পরীক্ষা খুব সন্নিকটে চলে এসেছে।
পড়াশোনা কেমন চলছে তোমার? ভালোভাবেই প্রস্তুতি নিচ্ছি।
তুমি কি কালকের ফুটবল ম্যাচটা সরাসরি মাঠে গিয়ে দেখেছিলে?
বাংলাদেশ ক্রিকেট দল আজ অসাধারণ দক্ষতা প্রদর্শন করে জয় লাভ করেছে।
তোমার প্রিয় শখ কী? বই পড়া এবং রবীন্দ্রসঙ্গীত শোনা আমার খুব পছন্দ।
সরাসরি কথা বললে মানুষের মধ্যকার সব ভুল বোঝাবুঝি দূর হয়ে যায়।
নিজের স্বাস্থ্যের প্রতি সবসময় যত্ন নেওয়া উচিত।
প্রতিদিন সকালে আধা ঘণ্টা হাঁটাহাঁটি করা স্বাস্থ্যের জন্য অত্যন্ত উপকারী।
খোদা হাফেজ, আবার দেখা হবে আমাদের আগামী সপ্তাহে।
আল্লাহ হাফেজ, সাবধানে যেও পথে।
ইনশাআল্লাহ আমাদের সব আশা ও স্বপ্ন পূরণ হবে।
আলহামদুলিল্লাহ আমি এখন আগের চেয়ে অনেক সুস্থ এবং ভালো আছি।
মাশাল্লাহ বাড়িটা দেখতে খুব সুন্দর এবং আকর্ষণীয় হয়েছে।
দেরি না করে এখনই আমাদের স্টেশন অভিমুখে রওনা হওয়া দরকার।
তুমি কোথায় আছো এখন? আমি অফিসের প্রধান ফটকের সামনে দাঁড়িয়ে আছি।
তোমার সাথে কথা বলে সত্যিই খুব ভালো লাগল।
কিছু মনে করো না, আমি কাজের চাপে একটু ব্যস্ত ছিলাম তখন।
মনে থাকবে তোমার এই মূল্যবান সহায়তা ও সুন্দর উপহারটি।
আজকের দিনটা ছিল একদম অন্যরকম আনন্দের এবং রোমাঞ্চকর।
টাকা পাঠিয়ে দিয়েছি তোমার ব্যাংক অ্যাকাউন্টে, চেক করে নিও।
ডকুমেন্টটি ডাউনলোড করে প্রিন্ট করে রেখো।
সবাই মিলে একসাথে কাজ করলে যেকোনো কঠিন কাজ সহজ হয়ে যায়।
"""


def sanitize_bengali_unicode(text: str) -> str:
    """Normalize Unicode to NFC form and sanitize ZWJ/ZWNJ artifacts."""
    # 1. Unicode NFC normalization
    text = unicodedata.normalize("NFC", text)

    # 2. Strip isolated or corrupted Zero-Width Non-Joiner (\u200C) and Joiner (\u200D)
    # Bengali correctly uses ZWJ/ZWNJ for specific ligature breaks (like খ্ণ্ড ত বা য-ফলা),
    # but strips duplicate or trailing zero-width characters.
    text = re.sub(r"[\u200c\u200d]{2,}", "\u200d", text)
    text = re.sub(r"[\u200c\u200d]+(?=[^\u0980-\u09ff]|$)", "", text)
    text = re.sub(r"(^[^\u0980-\u09ff]+)[\u200c\u200d]+", r"\1", text)

    return text


def clean_corpus_text(raw: str) -> str:
    """Clean raw article text: strip markup, templates, citations, and non-Bengali noise."""
    # Strip wiki section headings (e.g. == পরিচিতি ==)
    text = re.sub(r"={2,}[^=\n]+={2,}", "\n", raw)
    # Strip citation links e.g. [১], [২], [citation needed]
    text = re.sub(r"\[[^\]]*\]", "", text)
    # Strip URLs
    text = re.sub(r"https?://\S+", "", text)
    # Strip HTML tags
    text = re.sub(r"<[^>]+>", "", text)

    # Sentence boundary and line clean-up
    lines = []
    for line in text.splitlines():
        line = line.strip()
        if not line:
            continue

        # Count authentic Bengali characters (\u0980-\u09FF)
        bengali_chars = sum(1 for c in line if "\u0980" <= c <= "\u09ff")
        total_alpha = sum(1 for c in line if c.isalpha())

        # Keep lines where Bengali forms the vast majority and line length is meaningful
        if bengali_chars >= 15 and (total_alpha == 0 or bengali_chars / total_alpha >= 0.85):
            sanitized = sanitize_bengali_unicode(line)
            lines.append(sanitized)

    return "\n".join(lines)


def fetch_wikipedia_extract(title: str) -> str:
    """Fetch extract text for a given topic from Bengali Wikipedia API."""
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
            "User-Agent": "LekhaniCorpusPipeline/1.0 (https://github.com/sintaulsiam/lekhani)"
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=12) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            pages = data.get("query", {}).get("pages", {})
            for page in pages.values():
                return page.get("extract", "")
    except Exception as e:
        print(f"  [!] Failed to fetch '{title}': {e}", file=sys.stderr)
    return ""


def merge_local_directory(dir_path: Path) -> list:
    """Read and sanitize all text files from a local directory."""
    chunks = []
    if not dir_path.is_dir():
        print(f"  [!] Directory {dir_path} not found.", file=sys.stderr)
        return chunks

    txt_files = list(dir_path.rglob("*.txt")) + list(dir_path.rglob("*.corpus"))
    print(f"[*] Ingesting {len(txt_files)} local text files from {dir_path}...")
    for f in txt_files:
        try:
            content = f.read_text(encoding="utf-8", errors="replace")
            cleaned = clean_corpus_text(content)
            if cleaned:
                chunks.append(cleaned)
        except Exception as e:
            print(f"  [!] Error reading {f}: {e}", file=sys.stderr)
    return chunks


def extract_bz2_wikipedia_dump(dump_path: Path, max_articles: int = None) -> list:
    """Stream and extract plain text from bnwiki XML BZ2 dump."""
    import bz2
    import xml.etree.ElementTree as ET

    chunks = []
    print(f"[*] Streaming Wikipedia dump from {dump_path}...")
    count = 0
    with bz2.open(dump_path, "rt", encoding="utf-8", errors="replace") as f:
        # Simple fast streaming parser for <text>...</text> tags
        current_text = []
        in_text = False
        for line in f:
            if "<text" in line:
                in_text = True
                current_text = [line.split(">", 1)[-1]]
            elif "</text>" in line and in_text:
                current_text.append(line.split("</text>", 1)[0])
                in_text = False
                raw = "".join(current_text)
                cleaned = clean_corpus_text(raw)
                if len(cleaned) >= 100:
                    chunks.append(cleaned)
                    count += 1
                    if count % 1000 == 0:
                        print(f"  Extracted {count:,} articles...")
                    if max_articles and count >= max_articles:
                        break
                current_text = []
            elif in_text:
                current_text.append(line)
    print(f"[*] Dump extraction completed: {len(chunks):,} articles extracted.")
    return chunks


def main():
    parser = argparse.ArgumentParser(
        description="Lekhani Bengali Training Corpus Fetcher & Cleaner"
    )
    parser.add_argument(
        "--output",
        "-o",
        type=Path,
        default=Path("data/bengali_training_corpus.txt"),
        help="Target corpus text file (default: data/bengali_training_corpus.txt)",
    )
    parser.add_argument(
        "--merge-dir",
        "-m",
        type=Path,
        help="Optional local directory containing additional Bengali text files",
    )
    parser.add_argument(
        "--dump-bz2",
        type=Path,
        help="Optional Wikipedia XML bz2 dump file (e.g. bnwiki-latest-pages-articles.xml.bz2)",
    )
    parser.add_argument(
        "--skip-network",
        action="store_true",
        help="Skip online Wikipedia API fetch (useful when using --merge-dir or existing files)",
    )
    parser.add_argument(
        "--max-topics",
        type=int,
        default=len(TOPICS),
        help=f"Maximum topics to fetch from Wikipedia (default: {len(TOPICS)})",
    )

    args = parser.parse_args()

    args.output.parent.mkdir(parents=True, exist_ok=True)
    all_chunks = [clean_corpus_text(CONVERSATIONAL_DATA.strip())]

    # 1. Fetch online Wikipedia topics
    if not args.skip_network:
        topics_to_fetch = TOPICS[: args.max_topics]
        print(f"[*] Fetching Bengali corpus across {len(topics_to_fetch)} curated topics...")
        for idx, topic in enumerate(topics_to_fetch, 1):
            print(f"[{idx}/{len(topics_to_fetch)}] Fetching: {topic}...", end=" ", flush=True)
            raw = fetch_wikipedia_extract(topic)
            cleaned = clean_corpus_text(raw)
            char_count = len(cleaned)
            print(f"({char_count:,} chars)")
            if char_count > 0:
                all_chunks.append(cleaned)
            time.sleep(0.2)  # Courteous delay to respect Wikipedia rate limits

    # 2. Ingest local directory if specified
    if args.merge_dir:
        dir_chunks = merge_local_directory(args.merge_dir)
        all_chunks.extend(dir_chunks)

    # 3. Ingest bz2 dump if specified
    if args.dump_bz2:
        dump_chunks = extract_bz2_wikipedia_dump(args.dump_bz2)
        all_chunks.extend(dump_chunks)

    full_corpus = "\n\n".join(chunk for chunk in all_chunks if chunk)
    args.output.write_text(full_corpus, encoding="utf-8")

    word_count = len(full_corpus.split())
    line_count = len(full_corpus.splitlines())

    print("\n" + "═" * 60)
    print("  ✅ Bengali Training Corpus Successfully Built!")
    print(f"  📁 Output file:  {args.output.resolve()}")
    print(f"  📝 Total Words:  {word_count:,}")
    print(f"  🔤 Total Chars:  {len(full_corpus):,}")
    print(f"  📄 Total Lines:  {line_count:,}")
    print("═" * 60)
    print("\nTo compile and train the Lekhani binary model from this corpus:")
    print(f"  cargo run --release -p lekhani-cli -- train -i {args.output} -o data/dictionaries/bengali_lm.bin")
    print(f"  cargo run --release -p lekhani-cli -- eval\n")


if __name__ == "__main__":
    main()
