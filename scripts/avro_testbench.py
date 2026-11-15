#!/usr/bin/env python3
"""
Lekhani Avro Testbench & Interactive Playground
Provides a fast, zero-dependency, clean-state terminal REPL to test any Avro word,
phrase, or complex conjunct, with candidate inspections and automated test suites.
"""

import sys
import subprocess
import os

LEKHANI_BIN = "/mnt/data/lekhani/target/release/lekhani"

# Preset validation suites
PRESET_SUITES = {
    "1": ("Complex Conjuncts (যুক্তবর্ণ)", [
        ("shukhyo", "সূক্ষ্ম"),
        ("ushno", "উষ্ণ"),
        ("bramhoputro", "ব্রহ্মপুত্র"),
        ("attiyo", "আত্মীয়"),
        ("atmiyo", "আত্মীয়"),
        ("attotayee", "আততায়ী"),
        ("totwo", "তত্ত্ব"),
        ("ridoy", "হৃদয়"),
        ("akangkha", "আকাঙ্ক্ষা"),
        ("prosno", "প্রশ্ন"),
        ("ditio", "দ্বিতীয়"),
        ("tritio", "তৃতীয়"),
        ("srestho", "শ্রেষ্ঠ"),
        ("khomota", "ক্ষমতা"),
        ("totkalin", "তৎকালীন"),
        ("durghotona", "দুর্ঘটনা"),
        ("aschorjo", "আশ্চর্য"),
        ("sushrusa", "সুশ্রূষা"),
        ("singhashon", "সিংহাসন"),
    ]),
    "2": ("Prefix & Exact Match Lockout (No Longest-Word Hijack)", [
        ("desh", "দেশ"),
        ("din", "দিন"),
        ("rat", "রাত"),
        ("kaj", "কাজ"),
        ("bari", "বাড়ি"),
        ("bhalo", "ভালো"),
        ("shob", "সব"),
        ("pani", "পানি"),
        ("onno", "অন্য"),
        ("protidin", "প্রতিদিন"),
        ("shadharon", "সাধারণ"),
        ("bastob", "বাস্তব"),
    ]),
    "3": ("Morphology, Case Endings & Clitics", [
        ("boita", "বইটা"),
        ("deshe", "দেশে"),
        ("desher", "দেশের"),
        ("korcho", "করছো"),
        ("korbo", "করব"),
        ("amio", "আমিও"),
        ("shomoy", "সময়"),
        ("manusher", "মানুষের"),
    ]),
    "4": ("Full Sentences & Beam Decoding", [
        ("ami banglay gan gai", "আমি বাংলায় গান গাই"),
        ("ami bhat khai", "আমি ভাত খাই"),
        ("apni kemon achen", "আপনি কেমন আছেন"),
        ("tumi kemon acho", "তুমি কেমন আছো"),
        ("ami boi pori", "আমি বই পড়ি"),
        ("ami jama pori", "আমি জামা পরি"),
    ]),
}

def normalize_bengali(text):
    return text.replace("\u09AF\u09BC", "\u09DF").replace("\u09A1\u09BC", "\u09DC").replace("\u09A2\u09BC", "\u09DD")

def query_lekhani(term):
    try:
        out = subprocess.check_output([LEKHANI_BIN, "test", term], stderr=subprocess.STDOUT).decode("utf-8")
        lines = out.strip().split("\n")
        output_word = lines[1].split(":", 1)[1].strip() if len(lines) > 1 else ""
        cands_str = lines[2].split(":", 1)[1].strip() if len(lines) > 2 else "[]"
        import json
        try:
            candidates = json.loads(cands_str)
        except:
            candidates = [output_word]
        return output_word, candidates
    except Exception as e:
        return f"Error: {e}", []

def run_suite(suite_id):
    name, test_cases = PRESET_SUITES[suite_id]
    print(f"\n========================================================")
    print(f"  RUNNING BATTERY: {name}")
    print(f"========================================================")
    passed = 0
    total = len(test_cases)
    for inp, exp in test_cases:
        out_word, cands = query_lekhani(inp)
        norm_out = normalize_bengali(out_word)
        norm_exp = normalize_bengali(exp)
        if norm_out == norm_exp:
            print(f"  \033[92m✔ PASS\033[0m: {inp:18} -> {out_word}")
            passed += 1
        else:
            print(f"  \033[91m✘ FAIL\033[0m: {inp:18} -> {out_word} (expected: {exp})")
            print(f"         Candidates: {cands[:5]}")
    color = "\033[92m" if passed == total else "\033[93m"
    print(f"Score: {color}{passed}/{total} Passed ({(passed/total)*100:.1f}%)\033[0m\n")

def print_welcome():
    print("""
╔═══════════════════════════════════════════════════════════════════╗
║          Lekhani Avro Clean-State Interactive Playground          ║
║   Test any Roman input letter-by-letter, word, or phrase live!    ║
╚═══════════════════════════════════════════════════════════════════╝

Commands:
  :1          Run Suite 1: Complex Conjuncts (সূক্ষ্ম, উষ্ণ, ব্রহ্মপুত্র...)
  :2          Run Suite 2: Prefix & Longest-Word Lockout (দেশ, দিন, কাজ...)
  :3          Run Suite 3: Morphology & Clitics (বইটা, করছো, মানুষের...)
  :4          Run Suite 4: Full Sentences (আমি বাংলায় গান গাই...)
  :all        Run all 4 suites back-to-back
  :clear      Clear the screen
  :exit / :q  Exit playground

Type any word or phrase below to test in a 100% clean state:
""")

def main():
    if not os.path.exists(LEKHANI_BIN):
        print(f"Error: {LEKHANI_BIN} not found. Please run: cargo build --manifest-path /mnt/data/lekhani/Cargo.toml --release")
        sys.exit(1)

    # If run with command-line arguments, process them directly
    if len(sys.argv) > 1:
        arg = " ".join(sys.argv[1:])
        if arg in PRESET_SUITES:
            run_suite(arg)
            return
        elif arg == ":all":
            for sid in sorted(PRESET_SUITES.keys()):
                run_suite(sid)
            return
        else:
            out_word, cands = query_lekhani(arg)
            print(f"Input:       {arg}")
            print(f"Slot 0:      {out_word}")
            print(f"Candidates:  {cands}")
            return

    print_welcome()

    while True:
        try:
            line = input("\033[1mavro>\033[0m ").strip()
        except (EOFError, KeyboardInterrupt):
            print("\nExiting.")
            break

        if not line:
            continue

        if line in [":q", ":exit", "exit", "quit"]:
            break
        elif line == ":clear":
            os.system("clear")
            print_welcome()
            continue
        elif line in [":1", "1"]:
            run_suite("1")
        elif line in [":2", "2"]:
            run_suite("2")
        elif line in [":3", "3"]:
            run_suite("3")
        elif line in [":4", "4"]:
            run_suite("4")
        elif line == ":all":
            for sid in sorted(PRESET_SUITES.keys()):
                run_suite(sid)
        else:
            out_word, cands = query_lekhani(line)
            print(f"  \033[1;36mSlot 0 (Commit):\033[0m \033[1;32m{out_word}\033[0m")
            formatted = []
            for idx, c in enumerate(cands[:7]):
                tag = "\033[1;32m★\033[0m" if idx == 0 else f"{idx+1}."
                formatted.append(f"{tag} {c}")
            print(f"  \033[1;34mCandidate Strip:\033[0m " + "  │  ".join(formatted))
            print()

if __name__ == "__main__":
    main()

