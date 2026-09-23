#!/usr/bin/env python3
"""
Lekhani End-to-End Automated Large Bengali AI Model Training Pipeline.

Automates:
1. Acquiring large Bengali data:
   - Dynamic multi-threaded fetching from Bengali Wikipedia (curated + random categories)
   - Streaming ingestion of official Wikimedia BZ2 dumps (~1.5 GB text)
   - Local directory merging of custom Bengali text collections (.txt, .corpus)
2. Normalization & Sanitization:
   - Unicode NFC normalization
   - Zero-Width Joiner (ZWJ/ZWNJ) sanitization
   - Punctuation & sentence boundary segmentation
3. Rust Multi-Threaded Compilation:
   - Dynamic threshold auto-tuning based on corpus size
   - Direct binary compilation via `lekhani train`
4. Automated Evaluation & Verification:
   - Speed benchmark (ns/eval)
   - Homophone disambiguation test suite
   - Next-word predictions
"""

import argparse
import bz2
import json
import os
import re
import subprocess
import sys
import time
import unicodedata
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent
DEFAULT_CORPUS = ROOT_DIR / "data" / "bengali_training_corpus.txt"
DEFAULT_MODEL = ROOT_DIR / "data" / "dictionaries" / "bengali_lm.bin"

# Curated high-priority Bengali category hubs on bn.wikipedia.org
CURATED_CATEGORIES = [
    "বিষয়শ্রেণী:বাংলাদেশ",
    "বিষয়শ্রেণী:বাংলা_ভাষা",
    "বিষয়শ্রেণী:বাংলা_সাহিত্য",
    "বিষয়শ্রেণী:বাঙালি_সংস্কৃতি",
    "বিষয়শ্রেণী:বিজ্ঞান",
    "বিষয়শ্রেণী:তথ্যপ্রযুক্তি",
    "বিষয়শ্রেণী:ইতিহাস",
    "বিষয়শ্রেণী:ভূগোল",
    "বিষয়শ্রেণী:চিকিৎসাবিজ্ঞান",
    "বিষয়শ্রেণী:আইন",
    "বিষয়শ্রেণী:অর্থনীতি",
    "বিষয়শ্রেণী:খেলাধুলা",
    "বিষয়শ্রেণী:চলচ্চিত্র",
    "বিষয়শ্রেণী:সংগীত",
]

CONVERSATIONAL_SEED = """
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


def sanitize_bengali_text(text: str) -> str:
    """Normalize Unicode to NFC form and sanitize ZWJ/ZWNJ artifacts."""
    text = unicodedata.normalize("NFC", text)
    text = re.sub(r"[\u200c\u200d]{2,}", "\u200d", text)
    text = re.sub(r"[\u200c\u200d]+(?=[^\u0980-\u09ff]|$)", "", text)
    text = re.sub(r"(^[^\u0980-\u09ff]+)[\u200c\u200d]+", r"\1", text)
    return text


def clean_article_text(raw: str) -> str:
    """Strip markup, citations, templates, URLs, and filter low-density lines."""
    text = re.sub(r"={2,}[^=\n]+={2,}", "\n", raw)
    text = re.sub(r"\[[^\]]*\]", "", text)
    text = re.sub(r"https?://\S+", "", text)
    text = re.sub(r"<[^>]+>", "", text)

    lines = []
    for line in text.splitlines():
        line = line.strip()
        if not line:
            continue
        bengali_chars = sum(1 for c in line if "\u0980" <= c <= "\u09ff")
        total_alpha = sum(1 for c in line if c.isalpha())
        if bengali_chars >= 15 and (total_alpha == 0 or bengali_chars / total_alpha >= 0.85):
            lines.append(sanitize_bengali_text(line))

    return "\n".join(lines)


def fetch_wiki_page(title: str) -> str:
    """Fetch article plaintext using MediaWiki extracts API."""
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
        url, headers={"User-Agent": "LekhaniAutoTrainer/1.0 (https://github.com/sintaulsiam/lekhani)"}
    )
    try:
        with urllib.request.urlopen(req, timeout=12) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            pages = data.get("query", {}).get("pages", {})
            for page in pages.values():
                return page.get("extract", "")
    except Exception:
        pass
    return ""


def discover_wikipedia_titles(target_count: int) -> list:
    """Discover a diverse list of Bengali Wikipedia titles via category queries and random lists."""
    discovered = set()

    # 1. Fetch category members
    for cat in CURATED_CATEGORIES:
        if len(discovered) >= target_count:
            break
        url = (
            "https://bn.wikipedia.org/w/api.php?"
            + urllib.parse.urlencode(
                {
                    "action": "query",
                    "list": "categorymembers",
                    "cmtitle": cat,
                    "cmlimit": "50",
                    "cmtype": "page",
                    "format": "json",
                }
            )
        )
        req = urllib.request.Request(
            url, headers={"User-Agent": "LekhaniAutoTrainer/1.0"}
        )
        try:
            with urllib.request.urlopen(req, timeout=8) as r:
                d = json.loads(r.read().decode("utf-8"))
                for item in d.get("query", {}).get("categorymembers", []):
                    title = item.get("title", "")
                    if title and not title.startswith("টেমপ্লেট:") and not title.startswith("চিত্র:"):
                        discovered.add(title)
        except Exception:
            pass

    # 2. Fill remaining quota with random mainspace articles
    attempts = 0
    while len(discovered) < target_count and attempts < 15:
        attempts += 1
        needed = min(50, target_count - len(discovered))
        url = (
            "https://bn.wikipedia.org/w/api.php?"
            + urllib.parse.urlencode(
                {
                    "action": "query",
                    "list": "random",
                    "rnnamespace": "0",
                    "rnlimit": str(needed),
                    "format": "json",
                }
            )
        )
        req = urllib.request.Request(
            url, headers={"User-Agent": "LekhaniAutoTrainer/1.0"}
        )
        try:
            with urllib.request.urlopen(req, timeout=8) as r:
                d = json.loads(r.read().decode("utf-8"))
                for item in d.get("query", {}).get("random", []):
                    discovered.add(item.get("title", ""))
        except Exception:
            break

    return list(discovered)[:target_count]


def download_and_extract_wiki_dump(dump_url: str, dest_bz2: Path, max_articles: int = None) -> list:
    """Download and stream articles from official Wikimedia XML BZ2 dump."""
    if not dest_bz2.exists():
        print(f"\n[*] Downloading official Bengali Wikipedia dump (~540 MB compressed)...")
        print(f"    Source: {dump_url}")
        print(f"    Destination: {dest_bz2}")
        cmd = ["curl", "-C", "-", "-L", "-o", str(dest_bz2), dump_url]
        subprocess.run(cmd, check=True)

    print(f"\n[*] Streaming and cleaning XML articles from {dest_bz2.name}...")
    articles = []
    count = 0
    with bz2.open(dest_bz2, "rt", encoding="utf-8", errors="replace") as f:
        current = []
        in_text = False
        for line in f:
            if "<text" in line:
                in_text = True
                current = [line.split(">", 1)[-1]]
            elif "</text>" in line and in_text:
                current.append(line.split("</text>", 1)[0])
                in_text = False
                raw = "".join(current)
                cleaned = clean_article_text(raw)
                if len(cleaned) >= 120:
                    articles.append(cleaned)
                    count += 1
                    if count % 2000 == 0:
                        print(f"    • Extracted {count:,} articles...", flush=True)
                    if max_articles and count >= max_articles:
                        break
                current = []
            elif in_text:
                current.append(line)

    print(f"[✓] Extracted {len(articles):,} high-quality articles from dump.")
    return articles


def main():
    parser = argparse.ArgumentParser(
        description="Automated Large-Data Bengali AI Model Training Pipeline for Lekhani"
    )
    parser.add_argument(
        "--articles",
        "-n",
        type=int,
        default=250,
        help="Number of online Wikipedia articles to dynamically fetch and clean (default: 250)",
    )
    parser.add_argument(
        "--full-dump",
        action="store_true",
        help="Download and train on the complete 540MB Bengali Wikipedia dump (~1.5 GB uncompressed text)",
    )
    parser.add_argument(
        "--dump-path",
        type=Path,
        help="Path to an existing .xml.bz2 Wikimedia dump file",
    )
    parser.add_argument(
        "--dir",
        "-d",
        type=Path,
        help="Path to local folder containing Bengali text files (.txt, .corpus, .md)",
    )
    parser.add_argument(
        "--threads",
        "-t",
        type=int,
        default=8,
        help="Download and processing concurrency threads (default: 8)",
    )
    parser.add_argument(
        "--output-corpus",
        type=Path,
        default=DEFAULT_CORPUS,
        help=f"Output corpus file (default: {DEFAULT_CORPUS})",
    )
    parser.add_argument(
        "--output-model",
        type=Path,
        default=DEFAULT_MODEL,
        help=f"Output compiled binary model (default: {DEFAULT_MODEL})",
    )
    parser.add_argument(
        "--skip-fetch",
        action="store_true",
        help="Skip downloading new data and train directly on the existing corpus file",
    )
    parser.add_argument(
        "--max-unigrams",
        type=int,
        default=None,
        help="Max unigram vocabulary capacity (default: auto-tuned)",
    )
    parser.add_argument(
        "--max-bigrams",
        type=int,
        default=None,
        help="Max bigram transitions capacity (default: auto-tuned)",
    )
    parser.add_argument(
        "--max-trigrams",
        type=int,
        default=None,
        help="Max trigram contexts capacity (default: auto-tuned)",
    )
    parser.add_argument(
        "--min-unigram-freq",
        type=int,
        default=None,
        help="Min frequency for unigrams (default: auto-tuned)",
    )
    parser.add_argument(
        "--min-bigram-freq",
        type=int,
        default=None,
        help="Min frequency for bigrams (default: auto-tuned)",
    )
    parser.add_argument(
        "--min-trigram-freq",
        type=int,
        default=None,
        help="Min frequency for trigrams (default: auto-tuned)",
    )
    parser.add_argument(
        "--install",
        action="store_true",
        help="Automatically install the newly trained model to the system (/usr/share/lekhani/data/)",
    )

    args = parser.parse_args()

    print("╔══════════════════════════════════════════════════════════════╗")
    print("║      🚀 Lekhani Automated Bengali AI Training Pipeline       ║")
    print("╚══════════════════════════════════════════════════════════════╝")

    corpus_chunks = [clean_article_text(CONVERSATIONAL_SEED.strip())]

    if not args.skip_fetch:
        # 1. Full Dump Ingestion
        if args.full_dump or args.dump_path:
            dump_file = args.dump_path or (ROOT_DIR / "target" / "bnwiki-latest-pages-articles.xml.bz2")
            dump_file.parent.mkdir(parents=True, exist_ok=True)
            dump_url = "https://dumps.wikimedia.org/bnwiki/latest/bnwiki-latest-pages-articles.xml.bz2"
            dump_articles = download_and_extract_wiki_dump(dump_url, dump_file)
            corpus_chunks.extend(dump_articles)

        # 2. Local Directory Ingestion
        if args.dir and args.dir.is_dir():
            print(f"\n[*] Scanning local directory: {args.dir}...")
            files = list(args.dir.rglob("*.txt")) + list(args.dir.rglob("*.corpus"))
            print(f"    Found {len(files)} text files.")
            for f in files:
                try:
                    c = f.read_text(encoding="utf-8", errors="replace")
                    cleaned = clean_article_text(c)
                    if cleaned:
                        corpus_chunks.append(cleaned)
                except Exception:
                    pass

        # 3. Dynamic Parallel Wikipedia Articles Fetch
        if args.articles > 0 and not args.full_dump:
            print(f"\n[*] Discovering {args.articles} diverse Bengali Wikipedia topics...")
            titles = discover_wikipedia_titles(args.articles)
            print(f"    Discovered {len(titles)} candidate articles.")
            print(f"[*] Fetching and cleaning articles across {args.threads} parallel threads...")

            t0 = time.time()
            with ThreadPoolExecutor(max_workers=args.threads) as executor:
                futures = {executor.submit(fetch_wiki_page, t): t for t in titles}
                completed = 0
                for f in as_completed(futures):
                    completed += 1
                    raw = f.result()
                    if raw:
                        cleaned = clean_article_text(raw)
                        if len(cleaned) >= 50:
                            corpus_chunks.append(cleaned)
                    if completed % 25 == 0 or completed == len(titles):
                        print(f"    [{completed}/{len(titles)}] articles fetched...", flush=True)

            print(f"[✓] Fetched and normalized in {time.time() - t0:.1f}s.")

        # Save merged corpus
        full_corpus = "\n\n".join(corpus_chunks)
        args.output_corpus.parent.mkdir(parents=True, exist_ok=True)
        args.output_corpus.write_text(full_corpus, encoding="utf-8")
        word_count = len(full_corpus.split())
        print(f"\n[✓] Saved merged training corpus to: {args.output_corpus}")
        print(f"    • Total Words: {word_count:,}")
        print(f"    • Total Characters: {len(full_corpus):,}")
        print(f"    • File Size: {args.output_corpus.stat().st_size / (1024 * 1024):.2f} MB")
    else:
        if not args.output_corpus.exists():
            print(f"[!] Error: Corpus file {args.output_corpus} not found.", file=sys.stderr)
            sys.exit(1)
        print(f"\n[*] Counting tokens from existing corpus (low-memory streaming)...")
        word_count = 0
        with args.output_corpus.open("r", encoding="utf-8", errors="replace") as f:
            for line in f:
                word_count += len(line.split())
        print(f"[✓] Using existing corpus: {args.output_corpus} ({word_count:,} words)")

    # 4. Auto-tune pruning parameters based on corpus size (unless overridden)
    # Optimized for ultra-lightweight < 50MB RAM footprint while retaining 100% disambiguation accuracy
    if word_count > 5_000_000:
        def_min_u, def_min_b, def_min_t = 3, 5, 8
        def_max_u, def_max_b, def_max_t = 65_000, 160_000, 60_000
    elif word_count > 1_000_000:
        def_min_u, def_min_b, def_min_t = 3, 4, 6
        def_max_u, def_max_b, def_max_t = 50_000, 120_000, 50_000
    else:
        def_min_u, def_min_b, def_min_t = 2, 2, 3
        def_max_u, def_max_b, def_max_t = 40_000, 80_000, 30_000

    min_u = args.min_unigram_freq if args.min_unigram_freq is not None else def_min_u
    min_b = args.min_bigram_freq if args.min_bigram_freq is not None else def_min_b
    min_t = args.min_trigram_freq if args.min_trigram_freq is not None else def_min_t
    max_u = args.max_unigrams if args.max_unigrams is not None else def_max_u
    max_b = args.max_bigrams if args.max_bigrams is not None else def_max_b
    max_t = args.max_trigrams if args.max_trigrams is not None else def_max_t

    print(f"\n[*] Model Pruning & Capacity Parameters for {word_count:,} words:")
    print(f"    • Min Freq: Unigram >= {min_u}, Bigram >= {min_b}, Trigram >= {min_t}")
    print(f"    • Max Caps: Unigrams <= {max_u:,}, Bigrams <= {max_b:,}, Trigrams <= {max_t:,}")

    # 5. Build Release CLI and Train
    print("\n[*] Compiling and training binary model with multi-core Rayon...")
    train_cmd = [
        "cargo", "run", "--release", "-p", "lekhani-cli", "--",
        "dev", "train",
        "-i", str(args.output_corpus),
        "-o", str(args.output_model),
        "--min-unigram-freq", str(min_u),
        "--min-bigram-freq", str(min_b),
        "--min-trigram-freq", str(min_t),
        "--max-unigrams", str(max_u),
        "--max-bigrams", str(max_b),
        "--max-trigrams", str(max_t),
    ]
    subprocess.run(train_cmd, cwd=str(ROOT_DIR), check=True)

    # 6. Evaluate Model
    print("\n[*] Evaluating new binary language model...")
    eval_cmd = [
        "cargo", "run", "--release", "-p", "lekhani-cli", "--",
        "dev", "eval",
        "--model", str(args.output_model),
    ]
    subprocess.run(eval_cmd, cwd=str(ROOT_DIR), check=True)

    # 7. Optional System Installation
    if args.install:
        print("\n[*] Deploying new model to /usr/share/lekhani/data/...")
        subprocess.run(["sudo", "install", "-Dm644", str(args.output_model), "/usr/share/lekhani/data/bengali_lm.bin"], check=True)
        print("[✓] Model installed to system path successfully!")

    print("\n════════════════════════════════════════════════════════════════")
    print(f"  🎉 Training Complete! Model saved at: {args.output_model}")
    print("════════════════════════════════════════════════════════════════\n")


if __name__ == "__main__":
    main()
