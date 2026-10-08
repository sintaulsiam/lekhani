//! Comprehensive Emoji, Symbol & Kaomoji Catalog with Bilingual Search
//!
//! Provides curated, zero-tofu Unicode glyphs with Bengali and English keyword search,
//! categorised browsing, and grid generation for Slint UI.

use std::rc::Rc;
use crate::EmojiGridRow;

pub struct EmojiItem {
    pub glyph: &'static str,
    pub name: &'static str,
    pub tab: i32,        // 0: Emojis, 1: Symbols, 2: Kaomoji
    pub subcategory: i32,// Sub-category filter
    pub keywords: &'static [&'static str],
}

pub static CATALOG: &[EmojiItem] = &[
    // -------------------------------------------------------------------------
    // 1. EMOJIS: SMILEYS & EMOTIONS (tab: 0, subcat: 1)
    // -------------------------------------------------------------------------
    EmojiItem { glyph: "😀", name: "Grinning Face", tab: 0, subcategory: 1, keywords: &["smile", "happy", "grin", "হাসি", "খুশি", "আনন্দ"] },
    EmojiItem { glyph: "😃", name: "Smiling Face with Big Eyes", tab: 0, subcategory: 1, keywords: &["smile", "joy", "চোখ", "হাসি", "খুশি"] },
    EmojiItem { glyph: "😄", name: "Smiling Face with Smiling Eyes", tab: 0, subcategory: 1, keywords: &["smile", "laugh", "হাসিমুখ", "আনন্দ"] },
    EmojiItem { glyph: "😁", name: "Beaming Face with Smiling Eyes", tab: 0, subcategory: 1, keywords: &["teeth", "beam", "দাঁত", "হাসি"] },
    EmojiItem { glyph: "😆", name: "Grinning Squinting Face", tab: 0, subcategory: 1, keywords: &["laugh", "squint", "হাসাহাসি", "মজা"] },
    EmojiItem { glyph: "😅", name: "Grinning Face with Sweat", tab: 0, subcategory: 1, keywords: &["sweat", "nervous", "ঘাম", "হাসি"] },
    EmojiItem { glyph: "🤣", name: "Rolling on the Floor Laughing", tab: 0, subcategory: 1, keywords: &["rofl", "lol", "laugh", "হাসতে হাসতে শেষ"] },
    EmojiItem { glyph: "😂", name: "Face with Tears of Joy", tab: 0, subcategory: 1, keywords: &["tears", "joy", "cry", "কান্না হাসি", "মজা"] },
    EmojiItem { glyph: "🙂", name: "Slightly Smiling Face", tab: 0, subcategory: 1, keywords: &["smile", "slight", "মুচকি হাসি", "শান্ত"] },
    EmojiItem { glyph: "🙃", name: "Upside-Down Face", tab: 0, subcategory: 1, keywords: &["upside down", "silly", "উল্টো মুখ", "ফাজলামি"] },
    EmojiItem { glyph: "😉", name: "Winking Face", tab: 0, subcategory: 1, keywords: &["wink", "flirt", "চোখ টেপা", "ইশারা"] },
    EmojiItem { glyph: "😊", name: "Smiling Face with Smiling Eyes", tab: 0, subcategory: 1, keywords: &["blush", "proud", "লাজুক হাসি", "শান্তি"] },
    EmojiItem { glyph: "😇", name: "Smiling Face with Halo", tab: 0, subcategory: 1, keywords: &["angel", "halo", "innocent", "পবিত্র", "দেবদূত"] },
    EmojiItem { glyph: "🥰", name: "Smiling Face with Hearts", tab: 0, subcategory: 1, keywords: &["love", "hearts", "adore", "ভালোবাসা", "প্রেম"] },
    EmojiItem { glyph: "😍", name: "Heart Eyes", tab: 0, subcategory: 1, keywords: &["love", "crush", "heart eyes", "মুগ্ধ", "ভালোবাসা"] },
    EmojiItem { glyph: "🤩", name: "Star-Struck", tab: 0, subcategory: 1, keywords: &["star", "excited", "তারকা", "বিস্ময়"] },
    EmojiItem { glyph: "😘", name: "Face Blowing a Kiss", tab: 0, subcategory: 1, keywords: &["kiss", "love", "চুমু", "ভালোবাসা"] },
    EmojiItem { glyph: "😋", name: "Face Savoring Food", tab: 0, subcategory: 1, keywords: &["yum", "delicious", "tongue", "সুস্বাদু", "জিভ"] },
    EmojiItem { glyph: "😛", name: "Face with Tongue", tab: 0, subcategory: 1, keywords: &["tongue", "fun", "জিভ", "মজা"] },
    EmojiItem { glyph: "😜", name: "Winking Face with Tongue", tab: 0, subcategory: 1, keywords: &["wink", "crazy", "ফাজলামি", "মজা"] },
    EmojiItem { glyph: "🤪", name: "Zany Face", tab: 0, subcategory: 1, keywords: &["crazy", "goofy", "wild", "পাগলামি", "পাগল"] },
    EmojiItem { glyph: "🤑", name: "Money-Mouth Face", tab: 0, subcategory: 1, keywords: &["money", "rich", "dollar", "টাকা", "ধনী"] },
    EmojiItem { glyph: "🤗", name: "Hugging Face", tab: 0, subcategory: 1, keywords: &["hug", "warmth", "জড়ানো", "কোলাকুলি"] },
    EmojiItem { glyph: "🤭", name: "Face with Hand Over Mouth", tab: 0, subcategory: 1, keywords: &["giggle", "oops", "মুখ চাপা হাসি"] },
    EmojiItem { glyph: "🤫", name: "Shushing Face", tab: 0, subcategory: 1, keywords: &["shh", "quiet", "secret", "চুপ", "শান্ত"] },
    EmojiItem { glyph: "🤔", name: "Thinking Face", tab: 0, subcategory: 1, keywords: &["think", "ponder", "ভাবনা", "চিন্তা"] },
    EmojiItem { glyph: "🤐", name: "Zipper-Mouth Face", tab: 0, subcategory: 1, keywords: &["zipper", "silent", "তালা", "মুখ বন্ধ"] },
    EmojiItem { glyph: "🤨", name: "Face with Raised Eyebrow", tab: 0, subcategory: 1, keywords: &["eyebrow", "skeptical", "সন্দেহ", "ভ্রূ"] },
    EmojiItem { glyph: "😐", name: "Neutral Face", tab: 0, subcategory: 1, keywords: &["neutral", "meh", "সাধারণ", "নিরপেক্ষ"] },
    EmojiItem { glyph: "😑", name: "Expressionless Face", tab: 0, subcategory: 1, keywords: &["expressionless", "bored", "বিরক্ত", "ভাবলেশহীন"] },
    EmojiItem { glyph: "😏", name: "Smirking Face", tab: 0, subcategory: 1, keywords: &["smirk", "flirt", "বাঁকা হাসি", "শয়তানি"] },
    EmojiItem { glyph: "😒", name: "Unamused Face", tab: 0, subcategory: 1, keywords: &["unamused", "annoyed", "বিরক্ত", "অখুশি"] },
    EmojiItem { glyph: "🙄", name: "Face with Rolling Eyes", tab: 0, subcategory: 1, keywords: &["eye roll", "bored", "চোখ ঘোরানো", "ধুর"] },
    EmojiItem { glyph: "😬", name: "Grimacing Face", tab: 0, subcategory: 1, keywords: &["grimace", "awkward", "অস্বস্তি", "লজ্জা"] },
    EmojiItem { glyph: "🤥", name: "Lying Face", tab: 0, subcategory: 1, keywords: &["lie", "pinocchio", "মিথ্যা", "নাক"] },
    EmojiItem { glyph: "😌", name: "Relieved Face", tab: 0, subcategory: 1, keywords: &["relief", "peace", "স্বস্তি", "শান্তি"] },
    EmojiItem { glyph: "😔", name: "Pensive Face", tab: 0, subcategory: 1, keywords: &["sad", "pensive", "বিষণ্ণ", "হতাশ"] },
    EmojiItem { glyph: "😪", name: "Sleepy Face", tab: 0, subcategory: 1, keywords: &["sleepy", "tired", "ঘুম", "ক্লান্ত"] },
    EmojiItem { glyph: "🤤", name: "Drooling Face", tab: 0, subcategory: 1, keywords: &["drool", "hungry", "লালা", "লোভ"] },
    EmojiItem { glyph: "😴", name: "Sleeping Face", tab: 0, subcategory: 1, keywords: &["sleep", "zzz", "ঘুমানো", "বিশ্রাম"] },
    EmojiItem { glyph: "😷", name: "Face with Medical Mask", tab: 0, subcategory: 1, keywords: &["mask", "sick", "মাস্ক", "অসুখ"] },
    EmojiItem { glyph: "🤒", name: "Face with Thermometer", tab: 0, subcategory: 1, keywords: &["fever", "ill", "sick", "জ্বর", "অসুস্থ"] },
    EmojiItem { glyph: "🤕", name: "Face with Head-Bandage", tab: 0, subcategory: 1, keywords: &["hurt", "bandage", "আঘাত", "ব্যান্ডেজ"] },
    EmojiItem { glyph: "🤢", name: "Nauseated Face", tab: 0, subcategory: 1, keywords: &["nauseated", "gross", "বমি", "অসুস্থ"] },
    EmojiItem { glyph: "🤮", name: "Face Vomiting", tab: 0, subcategory: 1, keywords: &["vomit", "puke", "বমি করা", "ঘৃণা"] },
    EmojiItem { glyph: "🤧", name: "Sneezing Face", tab: 0, subcategory: 1, keywords: &["sneeze", "cold", "হাঁচি", "সর্দি"] },
    EmojiItem { glyph: "🥵", name: "Hot Face", tab: 0, subcategory: 1, keywords: &["hot", "heat", "sweat", "গরম", "ঘাম"] },
    EmojiItem { glyph: "🥶", name: "Cold Face", tab: 0, subcategory: 1, keywords: &["cold", "freezing", "ice", "ঠান্ডা", "শীত"] },
    EmojiItem { glyph: "🥴", name: "Woozy Face", tab: 0, subcategory: 1, keywords: &["woozy", "tipsy", "মাতাল", "ঘোর"] },
    EmojiItem { glyph: "😵", name: "Dizzy Face", tab: 0, subcategory: 1, keywords: &["dizzy", "faint", "মাথা ঘোরা", "অজ্ঞান"] },
    EmojiItem { glyph: "🤯", name: "Exploding Head", tab: 0, subcategory: 1, keywords: &["mind blown", "shock", "মাথা নষ্ট", "বিস্ফোরণ"] },
    EmojiItem { glyph: "🤠", name: "Cowboy Hat Face", tab: 0, subcategory: 1, keywords: &["cowboy", "hat", "কাউবয়", "টুপি"] },
    EmojiItem { glyph: "🥳", name: "Partying Face", tab: 0, subcategory: 1, keywords: &["party", "celebrate", "উৎসব", "উদযাপন"] },
    EmojiItem { glyph: "😎", name: "Smiling Face with Sunglasses", tab: 0, subcategory: 1, keywords: &["cool", "sunglasses", "স্মার্ট", "চশমা"] },
    EmojiItem { glyph: "🤓", name: "Nerd Face", tab: 0, subcategory: 1, keywords: &["nerd", "geek", "পড়ুয়া", "বুদ্ধিমান"] },
    EmojiItem { glyph: "🧐", name: "Face with Monocle", tab: 0, subcategory: 1, keywords: &["curious", "monocle", "তদন্ত", "চশমা"] },
    EmojiItem { glyph: "😕", name: "Confused Face", tab: 0, subcategory: 1, keywords: &["confused", "বিভ্রান্ত", "অনিশ্চিত"] },
    EmojiItem { glyph: "😟", name: "Worried Face", tab: 0, subcategory: 1, keywords: &["worried", "nervous", "চিন্তিত", "উদ্বিগ্ন"] },
    EmojiItem { glyph: "🙁", name: "Slightly Frowning Face", tab: 0, subcategory: 1, keywords: &["frown", "sad", "মন খারাপ", "অখুশি"] },
    EmojiItem { glyph: "😮", name: "Face with Open Mouth", tab: 0, subcategory: 1, keywords: &["wow", "surprise", "বিস্ময়", "হা"] },
    EmojiItem { glyph: "😯", name: "Hushed Face", tab: 0, subcategory: 1, keywords: &["hushed", "quiet", "হতবাক"] },
    EmojiItem { glyph: "😲", name: "Astonished Face", tab: 0, subcategory: 1, keywords: &["astonished", "shock", "চমকে যাওয়া"] },
    EmojiItem { glyph: "😳", name: "Flushed Face", tab: 0, subcategory: 1, keywords: &["flushed", "shy", "লজ্জিত", "লাল মুখ"] },
    EmojiItem { glyph: "🥺", name: "Pleading Face", tab: 0, subcategory: 1, keywords: &["pleading", "puppy eyes", "অনুরোধ", "মিনতি"] },
    EmojiItem { glyph: "😦", name: "Frowning Face with Open Mouth", tab: 0, subcategory: 1, keywords: &["frown", "shock", "হাঁ"] },
    EmojiItem { glyph: "😨", name: "Fearful Face", tab: 0, subcategory: 1, keywords: &["fear", "scared", "ভীত", "ভয়"] },
    EmojiItem { glyph: "😰", name: "Anxious Face with Sweat", tab: 0, subcategory: 1, keywords: &["anxious", "sweat", "উদ্বেগ", "ঘাম"] },
    EmojiItem { glyph: "😥", name: "Sad but Relieved Face", tab: 0, subcategory: 1, keywords: &["relieved", "sad", "কষ্ট", "স্বস্তি"] },
    EmojiItem { glyph: "😢", name: "Crying Face", tab: 0, subcategory: 1, keywords: &["cry", "tear", "sad", "কান্না", "অশ্রু"] },
    EmojiItem { glyph: "😭", name: "Loudly Crying Face", tab: 0, subcategory: 1, keywords: &["sob", "cry", "tears", "অঝোরে কান্না", "চিৎকার"] },
    EmojiItem { glyph: "😱", name: "Face Screaming in Fear", tab: 0, subcategory: 1, keywords: &["scream", "scared", "চিৎকার", "ভয়"] },
    EmojiItem { glyph: "😖", name: "Confounded Face", tab: 0, subcategory: 1, keywords: &["confounded", "pain", "যন্ত্রণা", "কষ্ট"] },
    EmojiItem { glyph: "😣", name: "Persevering Face", tab: 0, subcategory: 1, keywords: &["struggle", "ধৈর্য", "সংগ্রাম"] },
    EmojiItem { glyph: "😞", name: "Disappointed Face", tab: 0, subcategory: 1, keywords: &["disappointed", "sad", "হতাশ", "মন খারাপ"] },
    EmojiItem { glyph: "😓", name: "Downcast Face with Sweat", tab: 0, subcategory: 1, keywords: &["sweat", "tired", "ক্লান্ত", "ঘাম"] },
    EmojiItem { glyph: "😩", name: "Weary Face", tab: 0, subcategory: 1, keywords: &["weary", "tired", "ক্লান্তিকর", "ধুর"] },
    EmojiItem { glyph: "😫", name: "Tired Face", tab: 0, subcategory: 1, keywords: &["exhausted", "tired", "খুব ক্লান্ত"] },
    EmojiItem { glyph: "🥱", name: "Yawning Face", tab: 0, subcategory: 1, keywords: &["yawn", "sleepy", "হাই তোলা", "ঘুম"] },
    EmojiItem { glyph: "😤", name: "Face with Steam From Nose", tab: 0, subcategory: 1, keywords: &["steam", "proud", "anger", "রাগ", "জেদ"] },
    EmojiItem { glyph: "😡", name: "Enraged Face", tab: 0, subcategory: 1, keywords: &["angry", "rage", "রাগ", "ক্রোধ"] },
    EmojiItem { glyph: "😠", name: "Angry Face", tab: 0, subcategory: 1, keywords: &["mad", "angry", "রাগ", "গোস্বা"] },
    EmojiItem { glyph: "🤬", name: "Face with Symbols on Mouth", tab: 0, subcategory: 1, keywords: &["swearing", "curse", "গালি", "চরম রাগ"] },
    EmojiItem { glyph: "😈", name: "Smiling Face with Horns", tab: 0, subcategory: 1, keywords: &["devil", "evil", "শয়তান", "দুষ্টু"] },
    EmojiItem { glyph: "👿", name: "Angry Face with Horns", tab: 0, subcategory: 1, keywords: &["demon", "angry", "শয়তানি", "রাগ"] },
    EmojiItem { glyph: "💀", name: "Skull", tab: 0, subcategory: 1, keywords: &["dead", "skull", "কঙ্কাল", "মৃত"] },
    EmojiItem { glyph: "💩", name: "Pile of Poo", tab: 0, subcategory: 1, keywords: &["poop", "funny", "মল", "মজা"] },
    EmojiItem { glyph: "🤡", name: "Clown Face", tab: 0, subcategory: 1, keywords: &["clown", "funny", "জোকার", "বিদূষক"] },
    EmojiItem { glyph: "👻", name: "Ghost", tab: 0, subcategory: 1, keywords: &["ghost", "spooky", "ভূত", "প্রেত"] },
    EmojiItem { glyph: "👽", name: "Alien", tab: 0, subcategory: 1, keywords: &["alien", "ufo", "এলিয়েন", "মহাকাশ"] },
    EmojiItem { glyph: "🤖", name: "Robot", tab: 0, subcategory: 1, keywords: &["robot", "bot", "রোবট", "যন্ত্র"] },

    // -------------------------------------------------------------------------
    // 2. EMOJIS: GESTURES & HANDS (tab: 0, subcat: 2)
    // -------------------------------------------------------------------------
    EmojiItem { glyph: "👍", name: "Thumbs Up", tab: 0, subcategory: 2, keywords: &["thumbs up", "like", "yes", "লাইক", "পছন্দ", "ঠিক"] },
    EmojiItem { glyph: "👎", name: "Thumbs Down", tab: 0, subcategory: 2, keywords: &["thumbs down", "dislike", "no", "অপছন্দ", "না"] },
    EmojiItem { glyph: "👌", name: "OK Hand", tab: 0, subcategory: 2, keywords: &["ok", "perfect", "ঠিক আছে", "চমৎকার"] },
    EmojiItem { glyph: "✌", name: "Victory Hand", tab: 0, subcategory: 2, keywords: &["peace", "victory", "two", "শান্তি", "জয়", "বিজয়"] },
    EmojiItem { glyph: "🤞", name: "Crossed Fingers", tab: 0, subcategory: 2, keywords: &["luck", "hope", "শুভকামনা", "আশা"] },
    EmojiItem { glyph: "🤟", name: "Love-You Gesture", tab: 0, subcategory: 2, keywords: &["love you", "rock", "ভালোবাসা", "ইশারা"] },
    EmojiItem { glyph: "🤘", name: "Sign of the Horns", tab: 0, subcategory: 2, keywords: &["rock on", "metal", "রক", "সংগীত"] },
    EmojiItem { glyph: "🤙", name: "Call Me Hand", tab: 0, subcategory: 2, keywords: &["call me", "phone", "ফোন", "কল"] },
    EmojiItem { glyph: "👈", name: "Backhand Index Pointing Left", tab: 0, subcategory: 2, keywords: &["point left", "বাঁয়ে", "বাম"] },
    EmojiItem { glyph: "👉", name: "Backhand Index Pointing Right", tab: 0, subcategory: 2, keywords: &["point right", "ডানে", "ডান"] },
    EmojiItem { glyph: "👆", name: "Backhand Index Pointing Up", tab: 0, subcategory: 2, keywords: &["point up", "উপরে"] },
    EmojiItem { glyph: "👇", name: "Backhand Index Pointing Down", tab: 0, subcategory: 2, keywords: &["point down", "নিচে"] },
    EmojiItem { glyph: "☝", name: "Index Pointing Up", tab: 0, subcategory: 2, keywords: &["one", "up", "এক", "আল্লাহ", "উপরে"] },
    EmojiItem { glyph: "✋", name: "Raised Hand", tab: 0, subcategory: 2, keywords: &["stop", "high five", "হাত", "থামো"] },
    EmojiItem { glyph: "🤚", name: "Raised Back of Hand", tab: 0, subcategory: 2, keywords: &["backhand", "হাত"] },
    EmojiItem { glyph: "🖐", name: "Hand with Fingers Splayed", tab: 0, subcategory: 2, keywords: &["five", "hand", "পাঁচ", "হাত"] },
    EmojiItem { glyph: "🖖", name: "Vulcan Salute", tab: 0, subcategory: 2, keywords: &["vulcan", "salute", "স্যালুট"] },
    EmojiItem { glyph: "👋", name: "Waving Hand", tab: 0, subcategory: 2, keywords: &["wave", "hello", "bye", "বিদায়", "হ্যালো", "হাত নাড়া"] },
    EmojiItem { glyph: "👏", name: "Clapping Hands", tab: 0, subcategory: 2, keywords: &["clap", "bravo", "তালি", "অভিনন্দন"] },
    EmojiItem { glyph: "🙌", name: "Raising Hands", tab: 0, subcategory: 2, keywords: &["celebration", "praise", "হাত তোলা", "উল্লাস"] },
    EmojiItem { glyph: "👐", name: "Open Hands", tab: 0, subcategory: 2, keywords: &["open hands", "hug", "খোলা হাত"] },
    EmojiItem { glyph: "🤲", name: "Palms Up Together", tab: 0, subcategory: 2, keywords: &["dua", "prayer", "দোয়া", "প্রার্থনা", "মুনাজাত"] },
    EmojiItem { glyph: "🤝", name: "Handshake", tab: 0, subcategory: 2, keywords: &["handshake", "deal", "করমর্দন", "চুক্তি", "বন্ধুত্ব"] },
    EmojiItem { glyph: "🙏", name: "Folded Hands", tab: 0, subcategory: 2, keywords: &["pray", "thanks", "namaste", "ধন্যবাদ", "প্রণাম", "প্রার্থনা"] },
    EmojiItem { glyph: "✍", name: "Writing Hand", tab: 0, subcategory: 2, keywords: &["write", "pen", "লেখা", "কলম", "লেখনী"] },
    EmojiItem { glyph: "💅", name: "Nail Polish", tab: 0, subcategory: 2, keywords: &["nails", "beauty", "নখ", "সৌন্দর্য"] },
    EmojiItem { glyph: "🤳", name: "Selfie", tab: 0, subcategory: 2, keywords: &["selfie", "camera", "সেলফি", "ছবি"] },
    EmojiItem { glyph: "💪", name: "Flexed Biceps", tab: 0, subcategory: 2, keywords: &["strong", "muscle", "power", "শক্তি", "পেশী", "সাহস"] },
    EmojiItem { glyph: "👀", name: "Eyes", tab: 0, subcategory: 2, keywords: &["look", "see", "চোখ", "দেখা"] },
    EmojiItem { glyph: "👁", name: "Eye", tab: 0, subcategory: 2, keywords: &["eye", "vision", "চোখ", "দৃষ্টি"] },
    EmojiItem { glyph: "👅", name: "Tongue", tab: 0, subcategory: 2, keywords: &["tongue", "taste", "জিভ", "স্বাদ"] },
    EmojiItem { glyph: "👄", name: "Mouth", tab: 0, subcategory: 2, keywords: &["lips", "mouth", "ঠোঁট", "মুখ"] },

    // -------------------------------------------------------------------------
    // 3. EMOJIS: HEARTS & EMOTION (tab: 0, subcat: 3)
    // -------------------------------------------------------------------------
    EmojiItem { glyph: "❤", name: "Red Heart", tab: 0, subcategory: 3, keywords: &["love", "heart", "red", "হৃদয়", "ভালোবাসা", "প্রেম"] },
    EmojiItem { glyph: "🧡", name: "Orange Heart", tab: 0, subcategory: 3, keywords: &["orange heart", "কমলা হৃদয়"] },
    EmojiItem { glyph: "💛", name: "Yellow Heart", tab: 0, subcategory: 3, keywords: &["yellow heart", "হলুদ হৃদয়", "বন্ধুত্ব"] },
    EmojiItem { glyph: "💚", name: "Green Heart", tab: 0, subcategory: 3, keywords: &["green heart", "সবুজ হৃদয়", "প্রকৃতি"] },
    EmojiItem { glyph: "💙", name: "Blue Heart", tab: 0, subcategory: 3, keywords: &["blue heart", "নীল হৃদয়", "শান্তি"] },
    EmojiItem { glyph: "💜", name: "Purple Heart", tab: 0, subcategory: 3, keywords: &["purple heart", "বেগুনী হৃদয়"] },
    EmojiItem { glyph: "🤎", name: "Brown Heart", tab: 0, subcategory: 3, keywords: &["brown heart", "বাদামী হৃদয়"] },
    EmojiItem { glyph: "🖤", name: "Black Heart", tab: 0, subcategory: 3, keywords: &["black heart", "কালো হৃদয়"] },
    EmojiItem { glyph: "🤍", name: "White Heart", tab: 0, subcategory: 3, keywords: &["white heart", "সাদা হৃদয়", "পবিত্র"] },
    EmojiItem { glyph: "💔", name: "Broken Heart", tab: 0, subcategory: 3, keywords: &["broken heart", "heartbreak", "ভাঙ্গা হৃদয়", "কষ্ট"] },
    EmojiItem { glyph: "❣", name: "Heart Exclamation", tab: 0, subcategory: 3, keywords: &["heart mark", "বিস্ময় চিহ্ন"] },
    EmojiItem { glyph: "💕", name: "Two Hearts", tab: 0, subcategory: 3, keywords: &["two hearts", "দুই হৃদয়", "প্রেম"] },
    EmojiItem { glyph: "💞", name: "Revolving Hearts", tab: 0, subcategory: 3, keywords: &["revolving hearts", "ঘূর্ণায়মান হৃদয়"] },
    EmojiItem { glyph: "💓", name: "Beating Heart", tab: 0, subcategory: 3, keywords: &["heartbeat", "হৃদস্পন্দন"] },
    EmojiItem { glyph: "💗", name: "Growing Heart", tab: 0, subcategory: 3, keywords: &["growing heart", "ভালোবাসা বৃদ্ধি"] },
    EmojiItem { glyph: "💖", name: "Sparkling Heart", tab: 0, subcategory: 3, keywords: &["sparkle heart", "উজ্জ্বল হৃদয়"] },
    EmojiItem { glyph: "💘", name: "Heart with Arrow", tab: 0, subcategory: 3, keywords: &["cupid", "arrow", "তীরবিদ্ধ হৃদয়"] },
    EmojiItem { glyph: "💝", name: "Heart with Ribbon", tab: 0, subcategory: 3, keywords: &["gift heart", "উপহার হৃদয়"] },
    EmojiItem { glyph: "💟", name: "Heart Decoration", tab: 0, subcategory: 3, keywords: &["heart decoration", "হৃদয় নকশা"] },
    EmojiItem { glyph: "💌", name: "Love Letter", tab: 0, subcategory: 3, keywords: &["letter", "love letter", "প্রেমের চিঠি"] },
    EmojiItem { glyph: "🔥", name: "Fire", tab: 0, subcategory: 3, keywords: &["fire", "flame", "hot", "lit", "আগুন", "হট", "উত্তাপ"] },
    EmojiItem { glyph: "✨", name: "Sparkles", tab: 0, subcategory: 3, keywords: &["sparkle", "stars", "shine", "ঝলক", "উজ্জ্বল", "তারা"] },
    EmojiItem { glyph: "🌟", name: "Glowing Star", tab: 0, subcategory: 3, keywords: &["glow star", "উজ্জ্বল তারা"] },
    EmojiItem { glyph: "💥", name: "Collision", tab: 0, subcategory: 3, keywords: &["boom", "explosion", "বিস্ফোরণ", "ধামাকা"] },
    EmojiItem { glyph: "💢", name: "Anger Symbol", tab: 0, subcategory: 3, keywords: &["anger", "mad", "রাগ"] },
    EmojiItem { glyph: "💬", name: "Speech Balloon", tab: 0, subcategory: 3, keywords: &["chat", "message", "কথা", "বার্তা", "চ্যাট"] },
    EmojiItem { glyph: "💭", name: "Thought Balloon", tab: 0, subcategory: 3, keywords: &["think", "idea", "ভাবনা", "চিন্তা"] },

    // -------------------------------------------------------------------------
    // 4. EMOJIS: NATURE, ANIMALS & WEATHER (tab: 0, subcat: 4)
    // -------------------------------------------------------------------------
    EmojiItem { glyph: "🐱", name: "Cat Face", tab: 0, subcategory: 4, keywords: &["cat", "kitten", "বিড়াল", "বিড়ালছানা"] },
    EmojiItem { glyph: "🐶", name: "Dog Face", tab: 0, subcategory: 4, keywords: &["dog", "puppy", "কুকুর"] },
    EmojiItem { glyph: "🐭", name: "Mouse Face", tab: 0, subcategory: 4, keywords: &["mouse", "ইঁদুর"] },
    EmojiItem { glyph: "🐰", name: "Rabbit Face", tab: 0, subcategory: 4, keywords: &["rabbit", "bunny", "খরগোশ"] },
    EmojiItem { glyph: "🦊", name: "Fox", tab: 0, subcategory: 4, keywords: &["fox", "শিয়াল"] },
    EmojiItem { glyph: "🐻", name: "Bear", tab: 0, subcategory: 4, keywords: &["bear", "ভাল্লুক"] },
    EmojiItem { glyph: "🐼", name: "Panda", tab: 0, subcategory: 4, keywords: &["panda", "পান্ডা"] },
    EmojiItem { glyph: "🐨", name: "Koala", tab: 0, subcategory: 4, keywords: &["koala", "কোয়ালা"] },
    EmojiItem { glyph: "🐯", name: "Tiger Face", tab: 0, subcategory: 4, keywords: &["tiger", "বাঘ", "রয়েল বেঙ্গল"] },
    EmojiItem { glyph: "🦁", name: "Lion", tab: 0, subcategory: 4, keywords: &["lion", "সিংহ"] },
    EmojiItem { glyph: "🐮", name: "Cow Face", tab: 0, subcategory: 4, keywords: &["cow", "গরু"] },
    EmojiItem { glyph: "🐷", name: "Pig Face", tab: 0, subcategory: 4, keywords: &["pig", "শূকর"] },
    EmojiItem { glyph: "🐸", name: "Frog", tab: 0, subcategory: 4, keywords: &["frog", "ব্যাঙ"] },
    EmojiItem { glyph: "🐵", name: "Monkey Face", tab: 0, subcategory: 4, keywords: &["monkey", "বানর"] },
    EmojiItem { glyph: "🐔", name: "Chicken", tab: 0, subcategory: 4, keywords: &["chicken", "মুরগি"] },
    EmojiItem { glyph: "🐧", name: "Penguin", tab: 0, subcategory: 4, keywords: &["penguin", "পেঙ্গুইন"] },
    EmojiItem { glyph: "🐦", name: "Bird", tab: 0, subcategory: 4, keywords: &["bird", "পাখি"] },
    EmojiItem { glyph: "🦆", name: "Duck", tab: 0, subcategory: 4, keywords: &["duck", "হাঁস"] },
    EmojiItem { glyph: "🦅", name: "Eagle", tab: 0, subcategory: 4, keywords: &["eagle", "ঈগল"] },
    EmojiItem { glyph: "🦉", name: "Owl", tab: 0, subcategory: 4, keywords: &["owl", "পেঁচা"] },
    EmojiItem { glyph: "🐝", name: "Honeybee", tab: 0, subcategory: 4, keywords: &["bee", "honey", "মৌমাছি", "মধু"] },
    EmojiItem { glyph: "🦋", name: "Butterfly", tab: 0, subcategory: 4, keywords: &["butterfly", "প্রজাপতি"] },
    EmojiItem { glyph: "🐟", name: "Fish", tab: 0, subcategory: 4, keywords: &["fish", "মাছ"] },
    EmojiItem { glyph: "🐬", name: "Dolphin", tab: 0, subcategory: 4, keywords: &["dolphin", "ডলফিন"] },
    EmojiItem { glyph: "🐳", name: "Spouting Whale", tab: 0, subcategory: 4, keywords: &["whale", "তিমি"] },
    EmojiItem { glyph: "🐘", name: "Elephant", tab: 0, subcategory: 4, keywords: &["elephant", "হাতি"] },
    EmojiItem { glyph: "🐎", name: "Horse", tab: 0, subcategory: 4, keywords: &["horse", "ঘোড়া"] },
    EmojiItem { glyph: "🌸", name: "Cherry Blossom", tab: 0, subcategory: 4, keywords: &["cherry blossom", "flower", "ফুল", "চেরি"] },
    EmojiItem { glyph: "🌹", name: "Rose", tab: 0, subcategory: 4, keywords: &["rose", "flower", "গোলাপ", "ফুল"] },
    EmojiItem { glyph: "🌻", name: "Sunflower", tab: 0, subcategory: 4, keywords: &["sunflower", "সূর্যমুখী"] },
    EmojiItem { glyph: "🌺", name: "Hibiscus", tab: 0, subcategory: 4, keywords: &["hibiscus", "জবা ফুল"] },
    EmojiItem { glyph: "💐", name: "Bouquet", tab: 0, subcategory: 4, keywords: &["bouquet", "flowers", "তোড়া", "ফুলের তোড়া"] },
    EmojiItem { glyph: "🌱", name: "Seedling", tab: 0, subcategory: 4, keywords: &["seedling", "plant", "চারা", "গাছ"] },
    EmojiItem { glyph: "🌲", name: "Evergreen Tree", tab: 0, subcategory: 4, keywords: &["tree", "গাছ", "বন"] },
    EmojiItem { glyph: "🌴", name: "Palm Tree", tab: 0, subcategory: 4, keywords: &["palm tree", "তালগাছ", "নারকেল"] },
    EmojiItem { glyph: "🍀", name: "Four Leaf Clover", tab: 0, subcategory: 4, keywords: &["clover", "luck", "পাতা", "ভাগ্য"] },
    EmojiItem { glyph: "🍁", name: "Maple Leaf", tab: 0, subcategory: 4, keywords: &["maple leaf", "পাতা"] },
    EmojiItem { glyph: "🍂", name: "Fallen Leaf", tab: 0, subcategory: 4, keywords: &["leaf", "autumn", "শুকনো পাতা"] },
    EmojiItem { glyph: "☀", name: "Sun", tab: 0, subcategory: 4, keywords: &["sun", "sunny", "সূর্য", "রোদ"] },
    EmojiItem { glyph: "🌙", name: "Crescent Moon", tab: 0, subcategory: 4, keywords: &["moon", "night", "চাঁদ", "রাত"] },
    EmojiItem { glyph: "⭐", name: "Star", tab: 0, subcategory: 4, keywords: &["star", "তারা", "নক্ষত্র"] },
    EmojiItem { glyph: "☁", name: "Cloud", tab: 0, subcategory: 4, keywords: &["cloud", "weather", "মেঘ", "আকাশ"] },
    EmojiItem { glyph: "🌧", name: "Cloud with Rain", tab: 0, subcategory: 4, keywords: &["rain", "weather", "বৃষ্টি", "মেঘ"] },
    EmojiItem { glyph: "⛈", name: "Cloud with Lightning and Rain", tab: 0, subcategory: 4, keywords: &["thunderstorm", "ঝড়", "বজ্রপাত"] },
    EmojiItem { glyph: "❄", name: "Snowflake", tab: 0, subcategory: 4, keywords: &["snow", "cold", "বরফ", "তুষার"] },
    EmojiItem { glyph: "⚡", name: "High Voltage", tab: 0, subcategory: 4, keywords: &["electricity", "lightning", "বিদ্যুৎ", "বাজ"] },
    EmojiItem { glyph: "🌈", name: "Rainbow", tab: 0, subcategory: 4, keywords: &["rainbow", "রংধনু"] },

    // -------------------------------------------------------------------------
    // 5. EMOJIS: FOOD & OBJECTS (tab: 0, subcat: 5)
    // -------------------------------------------------------------------------
    EmojiItem { glyph: "🍏", name: "Green Apple", tab: 0, subcategory: 5, keywords: &["green apple", "আপেল", "সবুজ আপেল"] },
    EmojiItem { glyph: "🍎", name: "Red Apple", tab: 0, subcategory: 5, keywords: &["apple", "আপেল"] },
    EmojiItem { glyph: "🍌", name: "Banana", tab: 0, subcategory: 5, keywords: &["banana", "কলা"] },
    EmojiItem { glyph: "🍉", name: "Watermelon", tab: 0, subcategory: 5, keywords: &["watermelon", "তরমুজ"] },
    EmojiItem { glyph: "🍇", name: "Grapes", tab: 0, subcategory: 5, keywords: &["grapes", "আঙ্গুর"] },
    EmojiItem { glyph: "🍓", name: "Strawberry", tab: 0, subcategory: 5, keywords: &["strawberry", "স্ট্রবেরি"] },
    EmojiItem { glyph: "🥭", name: "Mango", tab: 0, subcategory: 5, keywords: &["mango", "আম"] },
    EmojiItem { glyph: "🍍", name: "Pineapple", tab: 0, subcategory: 5, keywords: &["pineapple", "আনারস"] },
    EmojiItem { glyph: "🥥", name: "Coconut", tab: 0, subcategory: 5, keywords: &["coconut", "নারকেল"] },
    EmojiItem { glyph: "🍔", name: "Hamburger", tab: 0, subcategory: 5, keywords: &["burger", "fast food", "বার্গার"] },
    EmojiItem { glyph: "🍟", name: "French Fries", tab: 0, subcategory: 5, keywords: &["fries", "ফ্রাই"] },
    EmojiItem { glyph: "🍕", name: "Pizza", tab: 0, subcategory: 5, keywords: &["pizza", "পিজ্জা"] },
    EmojiItem { glyph: "🥪", name: "Sandwich", tab: 0, subcategory: 5, keywords: &["sandwich", "স্যান্ডউইচ"] },
    EmojiItem { glyph: "🌮", name: "Taco", tab: 0, subcategory: 5, keywords: &["taco", "টাকো"] },
    EmojiItem { glyph: "🍜", name: "Steaming Bowl", tab: 0, subcategory: 5, keywords: &["noodles", "ramen", "নুডলস"] },
    EmojiItem { glyph: "🍚", name: "Cooked Rice", tab: 0, subcategory: 5, keywords: &["rice", "ভাত"] },
    EmojiItem { glyph: "🍰", name: "Shortcake", tab: 0, subcategory: 5, keywords: &["cake", "কেক"] },
    EmojiItem { glyph: "🎂", name: "Birthday Cake", tab: 0, subcategory: 5, keywords: &["birthday", "cake", "জন্মদিনের কেক"] },
    EmojiItem { glyph: "🍫", name: "Chocolate Bar", tab: 0, subcategory: 5, keywords: &["chocolate", "চকলেট"] },
    EmojiItem { glyph: "🍩", name: "Doughnut", tab: 0, subcategory: 5, keywords: &["donut", "ডোনাট"] },
    EmojiItem { glyph: "☕", name: "Hot Beverage", tab: 0, subcategory: 5, keywords: &["coffee", "tea", "কফি", "চা", "গরম পানীয়"] },
    EmojiItem { glyph: "🍵", name: "Teacup Without Handle", tab: 0, subcategory: 5, keywords: &["green tea", "চা", "সবুজ চা"] },
    EmojiItem { glyph: "🥤", name: "Cup with Straw", tab: 0, subcategory: 5, keywords: &["juice", "drink", "জুস", "শরবত"] },
    EmojiItem { glyph: "🎉", name: "Party Popper", tab: 0, subcategory: 5, keywords: &["party", "tada", "উৎসব", "অভিনন্দন", "উদযাপন"] },
    EmojiItem { glyph: "🎊", name: "Confetti Ball", tab: 0, subcategory: 5, keywords: &["confetti", "আনন্দ", "উৎসব"] },
    EmojiItem { glyph: "🎁", name: "Wrapped Gift", tab: 0, subcategory: 5, keywords: &["gift", "present", "উপহার"] },
    EmojiItem { glyph: "🏆", name: "Trophy", tab: 0, subcategory: 5, keywords: &["trophy", "winner", "পুরস্কার", "ট্রফি", "বিজয়"] },
    EmojiItem { glyph: "🎯", name: "Bullseye", tab: 0, subcategory: 5, keywords: &["target", "goal", "লক্ষ্য", "নিশানা"] },
    EmojiItem { glyph: "📱", name: "Mobile Phone", tab: 0, subcategory: 5, keywords: &["phone", "mobile", "ফোন", "মোবাইল"] },
    EmojiItem { glyph: "💻", name: "Laptop", tab: 0, subcategory: 5, keywords: &["computer", "laptop", "ল্যাপটপ", "কম্পিউটার"] },
    EmojiItem { glyph: "⌨", name: "Keyboard", tab: 0, subcategory: 5, keywords: &["keyboard", "typing", "কিবোর্ড", "টাইপ"] },
    EmojiItem { glyph: "💡", name: "Light Bulb", tab: 0, subcategory: 5, keywords: &["bulb", "idea", "বাতি", "বুদ্ধি", "আইডিয়া"] },
    EmojiItem { glyph: "📖", name: "Open Book", tab: 0, subcategory: 5, keywords: &["book", "read", "বই", "পড়া"] },
    EmojiItem { glyph: "📚", name: "Books", tab: 0, subcategory: 5, keywords: &["library", "books", "বইসমূহ", "লাইব্রেরি"] },
    EmojiItem { glyph: "💰", name: "Money Bag", tab: 0, subcategory: 5, keywords: &["money", "rich", "টাকা", "টাকার থলে", "সম্পদ"] },
    EmojiItem { glyph: "🪙", name: "Coin", tab: 0, subcategory: 5, keywords: &["coin", "পয়সা", "কয়েন"] },
    EmojiItem { glyph: "🔒", name: "Locked", tab: 0, subcategory: 5, keywords: &["lock", "secure", "তালা", "নিরাপদ"] },
    EmojiItem { glyph: "🔑", name: "Key", tab: 0, subcategory: 5, keywords: &["key", "চাবি"] },
    EmojiItem { glyph: "🛠", name: "Hammer and Wrench", tab: 0, subcategory: 5, keywords: &["tools", "repair", "যন্ত্রপাতি", "মেরামত"] },

    // -------------------------------------------------------------------------
    // 6. SYMBOLS: BENGALI, TYPOGRAPHICAL & MATH (tab: 1)
    // -------------------------------------------------------------------------
    EmojiItem { glyph: "৳", name: "Bengali Rupee / Taka Sign", tab: 1, subcategory: 1, keywords: &["taka", "bengali", "money", "currency", "টাকা", "পয়সা", "মুদ্রা"] },
    EmojiItem { glyph: "।", name: "Bengali Danda / Dari", tab: 1, subcategory: 1, keywords: &["dari", "danda", "fullstop", "দাঁড়ি", "বিরামচিহ্ন"] },
    EmojiItem { glyph: "॥", name: "Bengali Double Danda", tab: 1, subcategory: 1, keywords: &["double dari", "দ্বি-দাঁড়ি"] },
    EmojiItem { glyph: "৺", name: "Bengali Isshar Sign", tab: 1, subcategory: 1, keywords: &["isshar", "ঈশ্বর", "স্বর্গীয়"] },
    EmojiItem { glyph: "৲", name: "Bengali Rupee Mark", tab: 1, subcategory: 1, keywords: &["rupee mark", "মুদ্রা চিহ্ন"] },
    EmojiItem { glyph: "৴", name: "Bengali Currency Numerator One", tab: 1, subcategory: 1, keywords: &["ana", "আনা", "এক আনা"] },
    EmojiItem { glyph: "৵", name: "Bengali Currency Numerator Two", tab: 1, subcategory: 1, keywords: &["ana", "দুই আনা"] },
    EmojiItem { glyph: "৶", name: "Bengali Currency Numerator Three", tab: 1, subcategory: 1, keywords: &["ana", "তিন আনা"] },
    EmojiItem { glyph: "৷", name: "Bengali Currency Numerator Four", tab: 1, subcategory: 1, keywords: &["ana", "চার আনা", "গন্ডা"] },
    EmojiItem { glyph: "৸", name: "Bengali Currency Denominator Sixteen", tab: 1, subcategory: 1, keywords: &["ana", "ষোল আনা"] },
    EmojiItem { glyph: "৹", name: "Bengali Ganda Mark", tab: 1, subcategory: 1, keywords: &["ganda", "গন্ডা"] },
    EmojiItem { glyph: "—", name: "Em Dash", tab: 1, subcategory: 2, keywords: &["dash", "em dash", "ড্যাশ"] },
    EmojiItem { glyph: "–", name: "En Dash", tab: 1, subcategory: 2, keywords: &["en dash", "ড্যাশ"] },
    EmojiItem { glyph: "•", name: "Bullet", tab: 1, subcategory: 2, keywords: &["bullet", "dot", "বুলেট", "বিন্দু"] },
    EmojiItem { glyph: "©", name: "Copyright", tab: 1, subcategory: 2, keywords: &["copyright", "কপিরাইট", "স্বত্ব"] },
    EmojiItem { glyph: "®", name: "Registered Trademark", tab: 1, subcategory: 2, keywords: &["registered", "রেজিস্টার্ড"] },
    EmojiItem { glyph: "™", name: "Trademark", tab: 1, subcategory: 2, keywords: &["trademark", "ট্রেডমার্ক"] },
    EmojiItem { glyph: "§", name: "Section Sign", tab: 1, subcategory: 2, keywords: &["section", "ধারা", "আইন"] },
    EmojiItem { glyph: "¶", name: "Pilcrow / Paragraph Sign", tab: 1, subcategory: 2, keywords: &["paragraph", "অনুচ্ছেদ"] },
    EmojiItem { glyph: "‘", name: "Left Single Quotation Mark", tab: 1, subcategory: 2, keywords: &["quote", "উদ্ধৃতি"] },
    EmojiItem { glyph: "’", name: "Right Single Quotation Mark", tab: 1, subcategory: 2, keywords: &["quote", "উদ্ধৃতি"] },
    EmojiItem { glyph: "“", name: "Left Double Quotation Mark", tab: 1, subcategory: 2, keywords: &["double quote", "উদ্ধৃতি চিহ্ন"] },
    EmojiItem { glyph: "”", name: "Right Double Quotation Mark", tab: 1, subcategory: 2, keywords: &["double quote", "উদ্ধৃতি চিহ্ন"] },
    EmojiItem { glyph: "…", name: "Horizontal Ellipsis", tab: 1, subcategory: 2, keywords: &["ellipsis", "dots", "ডট ডট"] },
    EmojiItem { glyph: "°", name: "Degree Sign", tab: 1, subcategory: 3, keywords: &["degree", "temperature", "ডিগ্রি", "তাপমাত্রা"] },
    EmojiItem { glyph: "±", name: "Plus-Minus Sign", tab: 1, subcategory: 3, keywords: &["plus minus", "যোগ বিয়োগ"] },
    EmojiItem { glyph: "×", name: "Multiplication Sign", tab: 1, subcategory: 3, keywords: &["multiply", "times", "গুণ"] },
    EmojiItem { glyph: "÷", name: "Division Sign", tab: 1, subcategory: 3, keywords: &["divide", "ভাগ"] },
    EmojiItem { glyph: "≠", name: "Not Equal To", tab: 1, subcategory: 3, keywords: &["not equal", "অসমান"] },
    EmojiItem { glyph: "≈", name: "Almost Equal To", tab: 1, subcategory: 3, keywords: &["approx", "প্রায় সমান"] },
    EmojiItem { glyph: "≤", name: "Less-Than or Equal To", tab: 1, subcategory: 3, keywords: &["less equal", "ছোট বা সমান"] },
    EmojiItem { glyph: "≥", name: "Greater-Than or Equal To", tab: 1, subcategory: 3, keywords: &["greater equal", "বড় বা সমান"] },
    EmojiItem { glyph: "√", name: "Square Root", tab: 1, subcategory: 3, keywords: &["square root", "বর্গমূল", "রুট"] },
    EmojiItem { glyph: "∞", name: "Infinity", tab: 1, subcategory: 3, keywords: &["infinity", "অসীম"] },
    EmojiItem { glyph: "µ", name: "Micro Sign", tab: 1, subcategory: 3, keywords: &["micro", "মাইক্রো"] },
    EmojiItem { glyph: "%", name: "Percent Sign", tab: 1, subcategory: 3, keywords: &["percent", "শতকরা", "শতাংশ"] },
    EmojiItem { glyph: "‰", name: "Per Mille Sign", tab: 1, subcategory: 3, keywords: &["per mille", "হাজারে"] },
    EmojiItem { glyph: "π", name: "Pi", tab: 1, subcategory: 3, keywords: &["pi", "পাই", "গণিত"] },
    EmojiItem { glyph: "∑", name: "Summation", tab: 1, subcategory: 3, keywords: &["sum", "যোগফল"] },
    EmojiItem { glyph: "∆", name: "Increment / Delta", tab: 1, subcategory: 3, keywords: &["delta", "ডেল্টা"] },
    EmojiItem { glyph: "∫", name: "Integral", tab: 1, subcategory: 3, keywords: &["integral", "সমাকলন"] },
    EmojiItem { glyph: "←", name: "Leftwards Arrow", tab: 1, subcategory: 4, keywords: &["arrow left", "বাম তীর"] },
    EmojiItem { glyph: "→", name: "Rightwards Arrow", tab: 1, subcategory: 4, keywords: &["arrow right", "ডান তীর"] },
    EmojiItem { glyph: "↑", name: "Upwards Arrow", tab: 1, subcategory: 4, keywords: &["arrow up", "উপরের তীর"] },
    EmojiItem { glyph: "↓", name: "Downwards Arrow", tab: 1, subcategory: 4, keywords: &["arrow down", "নিচের তীর"] },
    EmojiItem { glyph: "↔", name: "Left Right Arrow", tab: 1, subcategory: 4, keywords: &["left right", "উভয়মুখী তীর"] },
    EmojiItem { glyph: "↕", name: "Up Down Arrow", tab: 1, subcategory: 4, keywords: &["up down", "লম্বালম্বি তীর"] },
    EmojiItem { glyph: "⇒", name: "Rightwards Double Arrow", tab: 1, subcategory: 4, keywords: &["double arrow", "নির্দেশক"] },
    EmojiItem { glyph: "⇔", name: "Left Right Double Arrow", tab: 1, subcategory: 4, keywords: &["if and only if", "সমতুল্য"] },
    EmojiItem { glyph: "✓", name: "Check Mark", tab: 1, subcategory: 5, keywords: &["check", "tick", "correct", "টিক", "ঠিক", "সঠিক"] },
    EmojiItem { glyph: "✕", name: "Multiplication X / Cross", tab: 1, subcategory: 5, keywords: &["cross", "wrong", "ভুল", "কাটা", "ক্রস"] },
    EmojiItem { glyph: "⚠", name: "Warning", tab: 1, subcategory: 5, keywords: &["warning", "caution", "সতর্কতা", "বিপদ"] },
    EmojiItem { glyph: "★", name: "Black Star", tab: 1, subcategory: 5, keywords: &["star", "তারা", "স্টার"] },
    EmojiItem { glyph: "☆", name: "White Star", tab: 1, subcategory: 5, keywords: &["star outline", "ফাঁকা তারা"] },
    EmojiItem { glyph: "✦", name: "Black Four Pointed Star", tab: 1, subcategory: 5, keywords: &["four star", "উজ্জ্বল"] },
    EmojiItem { glyph: "✧", name: "White Four Pointed Star", tab: 1, subcategory: 5, keywords: &["four star outline"] },
    EmojiItem { glyph: "●", name: "Black Circle", tab: 1, subcategory: 5, keywords: &["circle", "বৃত্ত", "গোল"] },
    EmojiItem { glyph: "○", name: "White Circle", tab: 1, subcategory: 5, keywords: &["circle outline", "ফাঁকা বৃত্ত"] },
    EmojiItem { glyph: "■", name: "Black Square", tab: 1, subcategory: 5, keywords: &["square", "বর্গ", "চারকোনা"] },
    EmojiItem { glyph: "□", name: "White Square", tab: 1, subcategory: 5, keywords: &["square outline", "ফাঁকা বর্গ"] },

    // -------------------------------------------------------------------------
    // 7. KAOMOJI (tab: 2)
    // -------------------------------------------------------------------------
    EmojiItem { glyph: "(◕‿◕)", name: "Cute Happy Smile", tab: 2, subcategory: 1, keywords: &["cute", "smile", "happy", "হাসি", "কিউট"] },
    EmojiItem { glyph: "(*^‿^*)", name: "Joyful Blushing", tab: 2, subcategory: 1, keywords: &["joy", "blush", "আনন্দ", "খুশি"] },
    EmojiItem { glyph: "(｡♥‿♥｡)", name: "In Love", tab: 2, subcategory: 1, keywords: &["love", "heart", "ভালোবাসা", "প্রেম"] },
    EmojiItem { glyph: "(づ｡◕‿‿◕｡)づ", name: "Giving a Hug", tab: 2, subcategory: 1, keywords: &["hug", "cuddle", "কোলাকুলি", "জড়ানো"] },
    EmojiItem { glyph: "¯\\_(ツ)_/¯", name: "Shrug", tab: 2, subcategory: 2, keywords: &["shrug", "whatever", "dunno", "শ্ৰাগ", "জানিনা"] },
    EmojiItem { glyph: "(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧", name: "Excited Magic", tab: 2, subcategory: 1, keywords: &["sparkle", "magic", "জাদু", "উত্তেজিত"] },
    EmojiItem { glyph: "(✿◠‿◠)", name: "Flower Smile", tab: 2, subcategory: 1, keywords: &["flower", "sweet", "ফুল", "মিষ্টি"] },
    EmojiItem { glyph: "٩(◕‿◕｡)۶", name: "Cheering Joy", tab: 2, subcategory: 1, keywords: &["cheer", "hooray", "উল্লাস", "জয়"] },
    EmojiItem { glyph: "(¬‿¬)", name: "Smirk", tab: 2, subcategory: 2, keywords: &["smirk", "sly", "বাঁকা হাসি", "শয়তানি"] },
    EmojiItem { glyph: "(•‿•)", name: "Simple Smile", tab: 2, subcategory: 1, keywords: &["smile", "simple", "সহজ হাসি"] },
    EmojiItem { glyph: "(´∀｀)♡", name: "Warm Affection", tab: 2, subcategory: 1, keywords: &["love", "warm", "স্নেহ", "মায়া"] },
    EmojiItem { glyph: "(ര‿ര)", name: "Curious Eyes", tab: 2, subcategory: 2, keywords: &["curious", "stare", "কৌতূহল"] },
    EmojiItem { glyph: "(╥﹏╥)", name: "Crying Tears", tab: 2, subcategory: 3, keywords: &["cry", "sad", "tears", "কান্না", "কষ্ট"] },
    EmojiItem { glyph: "(╯°□°）╯︵ ┻━┻", name: "Table Flip Rage", tab: 2, subcategory: 3, keywords: &["flip table", "rage", "angry", "রাগ", "টেবিল উল্টানো"] },
    EmojiItem { glyph: "(o_O)", name: "Confused Stare", tab: 2, subcategory: 3, keywords: &["confused", "shock", "বিস্ময়", "হতবাক"] },
    EmojiItem { glyph: "(ง •̀_•́)ง", name: "Ready to Fight", tab: 2, subcategory: 3, keywords: &["fight", "determined", "যুদ্ধ", "প্রস্তুত"] },
    EmojiItem { glyph: "( ͡° ͜ʖ ͡°)", name: "Lenny Face", tab: 2, subcategory: 2, keywords: &["lenny", "mischief", "লেনি", "দুষ্টুমি"] },
    EmojiItem { glyph: "ฅ^•ﻌ•^ฅ", name: "Cat Kitty", tab: 2, subcategory: 2, keywords: &["cat", "kitty", "বিড়াল", "বিড়ালছানা"] },
    EmojiItem { glyph: "( ˘ ³˘)♥", name: "Blowing a Kiss", tab: 2, subcategory: 1, keywords: &["kiss", "love", "চুমু", "ভালোবাসা"] },
    EmojiItem { glyph: "( •̀ᴗ•́ )و", name: "Yes We Can", tab: 2, subcategory: 1, keywords: &["yes", "success", "সফলতা", "দৃঢ়"] },
    EmojiItem { glyph: "(T_T)", name: "Crying Stream", tab: 2, subcategory: 3, keywords: &["cry", "sad", "কান্না"] },
    EmojiItem { glyph: "ಠ_ಠ", name: "Disapproval Look", tab: 2, subcategory: 3, keywords: &["look of disapproval", "বিরক্ত", "দৃষ্টি"] },
    EmojiItem { glyph: "(｡•́︿•̀｡)", name: "Pout Sad", tab: 2, subcategory: 3, keywords: &["sad", "pout", "অভিমান", "মন খারাপ"] },
    EmojiItem { glyph: "(>_<)", name: "Frustrated Pain", tab: 2, subcategory: 3, keywords: &["frustrated", "pain", "যন্ত্রণা", "বিরক্তি"] },
    EmojiItem { glyph: "(≧◡≦)", name: "Happy Cheerful", tab: 2, subcategory: 1, keywords: &["happy", "cheerful", "খুশি", "আনন্দিত"] },
];

pub fn search(query: &str, tab: i32, subcategory: i32) -> Vec<&'static str> {
    let q = query.trim().to_lowercase();
    let mut results = Vec::new();

    for item in CATALOG {
        if item.tab != tab {
            continue;
        }

        if !q.is_empty() {
            let match_glyph = item.glyph.contains(&q);
            let match_name = item.name.to_lowercase().contains(&q);
            let match_keyword = item.keywords.iter().any(|k| k.to_lowercase().contains(&q));

            if match_glyph || match_name || match_keyword {
                results.push(item.glyph);
            }
        } else {
            if subcategory == 0 || item.subcategory == subcategory {
                results.push(item.glyph);
            }
        }
    }

    results
}

pub fn to_grid_rows(glyphs: &[&str], cols: usize) -> Vec<EmojiGridRow> {
    let mut rows = Vec::new();
    let chunk_size = if cols == 0 { 10 } else { cols };

    for chunk in glyphs.chunks(chunk_size) {
        let items: Vec<slint::SharedString> = chunk.iter().map(|&s| s.into()).collect();
        rows.push(EmojiGridRow {
            items: Rc::new(slint::VecModel::from(items)).into(),
        });
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji_search_bengali_and_english() {
        let results_en = search("smile", 0, 0);
        assert!(!results_en.is_empty());
        assert!(results_en.contains(&"😀"));

        let results_bn = search("হাসি", 0, 0);
        assert!(!results_bn.is_empty());
        assert!(results_bn.contains(&"😀"));
    }

    #[test]
    fn test_symbol_search_taka() {
        let results = search("taka", 1, 0);
        assert!(!results.is_empty());
        assert!(results.contains(&"৳"));

        let results_bn = search("টাকা", 1, 0);
        assert!(!results_bn.is_empty());
        assert!(results_bn.contains(&"৳"));
    }

    #[test]
    fn test_kaomoji_search() {
        let results = search("shrug", 2, 0);
        assert!(!results.is_empty());
        assert!(results.contains(&"¯\\_(ツ)_/¯"));
    }

    #[test]
    fn test_no_variation_selector_in_catalog() {
        // Guarantee zero-tofu: none of the glyphs should contain \u{fe0f}
        for item in CATALOG {
            assert!(
                !item.glyph.contains('\u{fe0f}'),
                "Glyph {} in category {} contains variation selector 16 (\\u{{fe0f}}) which causes tofu boxes on Linux!",
                item.glyph,
                item.name
            );
        }
    }

    #[test]
    fn test_to_grid_rows() {
        let glyphs = vec!["😀", "😃", "😄", "😁", "😆", "😅", "😂", "🤣", "😊", "😇", "🥰"];
        let rows = to_grid_rows(&glyphs, 5);
        assert_eq!(rows.len(), 3);
    }
}
