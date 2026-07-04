// Simple VNI Vietnamese Parser & State Machine for the VietC Terminal simulator
// It processes VNI input (numbers 1-9 for diacritics) and returns the converted string
// and logs detailing the State Machine transitions and Token-Level Diffing.

interface VniStateResult {
  text: string;
  logs: string[];
  state: string; // S0, S1, S2, S3
}

// Maps for tone accents on vowels
const ACUTE = '\u0301'; // Sắc (1)
const GRAVE = '\u0300'; // Huyền (2)
const HOOK = '\u0309';  // Hỏi (3)
const TILDE = '\u0303'; // Ngã (4)
const DOT = '\u0323';   // Nặng (5)

const TONES_MAP: Record<string, string> = {
  '1': ACUTE,
  '2': GRAVE,
  '3': HOOK,
  '4': TILDE,
  '5': DOT,
};

// Maps for letter modifiers
// 6 -> â, ê, ô
// 7 -> ơ, ư
// 8 -> ă
// 9 -> đ
const MOD_6: Record<string, string> = {
  'a': 'â', 'A': 'Â',
  'e': 'ê', 'E': 'Ê',
  'o': 'ô', 'O': 'Ô',
};

const MOD_7: Record<string, string> = {
  'o': 'ơ', 'O': 'Ơ',
  'u': 'ư', 'U': 'Ư',
};

const MOD_8: Record<string, string> = {
  'a': 'ă', 'A': 'Ă',
};

/**
 * Normalizes combining diacritics to standard precomposed Vietnamese characters.
 */
function normalizeVietnamese(text: string): string {
  return text.normalize('NFC');
}

/**
 * Parse a full sentence/text typed in VNI.
 * E.g., "tieengs vieetj" -> "tiếng việt"
 * VNI: "vietetj" or "viet1" -> "viết"
 * Let's process word by word.
 */
export function parseVni(inputText: string): VniStateResult {
  const words = inputText.split(' ');
  const processedWords: string[] = [];
  const logs: string[] = [];
  let currentState = 'S0';

  for (let w = 0; w < words.length; w++) {
    const word = words[w];
    if (!word) {
      processedWords.push('');
      continue;
    }

    let resultWord = '';
    let tone: string | null = null;
    let dStroke = false;
    
    // We will build the word character-by-character
    for (let i = 0; i < word.length; i++) {
      const char = word[i];

      // Check for Đ (9)
      if (char === '9') {
        const lastChar = resultWord[resultWord.length - 1];
        if (lastChar === 'd' || lastChar === 'D') {
          resultWord = resultWord.slice(0, -1) + (lastChar === 'd' ? 'đ' : 'Đ');
          dStroke = true;
          currentState = 'S3';
          logs.push(`[uinput / S3] Nhận phím '9': Chuyển đổi phụ âm '${lastChar}' -> 'đ' (Độ trễ: 0ms)`);
        } else {
          resultWord += '9';
          logs.push(`[uinput / S0] Nhận phím '9': Không khớp phụ âm d/D, giữ nguyên chữ '9'`);
        }
        continue;
      }

      // Check for circumflex â, ê, ô (6)
      if (char === '6') {
        // Find last matching vowel in resultWord to apply modifier
        let applied = false;
        for (let j = resultWord.length - 1; j >= 0; j--) {
          const c = resultWord[j];
          if (MOD_6[c]) {
            resultWord = resultWord.substring(0, j) + MOD_6[c] + resultWord.substring(j + 1);
            applied = true;
            currentState = 'S3';
            logs.push(`[uinput / S3] Nhận phím '6': Thêm mũ ô/ê/â cho '${c}' -> '${MOD_6[c]}' (Độ trễ: 0ms)`);
            break;
          }
        }
        if (!applied) {
          resultWord += '6';
          logs.push(`[uinput / S0] Nhận phím '6': Không tìm thấy nguyên âm thích hợp để đội mũ (giữ nguyên '6')`);
        }
        continue;
      }

      // Check for horn ơ, ư (7)
      if (char === '7') {
        let applied = false;
        for (let j = resultWord.length - 1; j >= 0; j--) {
          const c = resultWord[j];
          if (MOD_7[c]) {
            resultWord = resultWord.substring(0, j) + MOD_7[c] + resultWord.substring(j + 1);
            // If 'o'->'ơ' preceded by 'u', merge to 'ươ' (standard VNI digraph)
            if (MOD_7[c] === 'ơ' && j > 0 && (resultWord[j-1] === 'u' || resultWord[j-1] === 'U')) {
              const prefix = resultWord.substring(0, j - 1);
              const suffix = resultWord.substring(j + 1);
              resultWord = prefix + (resultWord[j-1] === 'U' ? 'Ươ' : 'ươ') + suffix;
            }
            applied = true;
            currentState = 'S3';
            logs.push(`[uinput / S3] Nhận phím '7': Thêm râu ơ/ư cho '${c}' -> '${MOD_7[c]}' (Độ trễ: 0ms)`);
            break;
          }
        }
        if (!applied) {
          resultWord += '7';
          logs.push(`[uinput / S0] Nhận phím '7': Không tìm thấy nguyên âm o/u để thêm râu`);
        }
        continue;
      }

      // Check for breve ă (8)
      if (char === '8') {
        let applied = false;
        for (let j = resultWord.length - 1; j >= 0; j--) {
          const c = resultWord[j];
          if (MOD_8[c]) {
            resultWord = resultWord.substring(0, j) + MOD_8[c] + resultWord.substring(j + 1);
            applied = true;
            currentState = 'S3';
            logs.push(`[uinput / S3] Nhận phím '8': Thêm á cho '${c}' -> '${MOD_8[c]}' (Độ trễ: 0ms)`);
            break;
          }
        }
        if (!applied) {
          resultWord += '8';
          logs.push(`[uinput / S0] Nhận phím '8': Không tìm thấy nguyên âm a để chuyển thành ă`);
        }
        continue;
      }

      // Check for tones (1, 2, 3, 4, 5)
      if (TONES_MAP[char]) {
        tone = TONES_MAP[char];
        currentState = 'S2';
        const toneNames: Record<string, string> = { '1': 'Sắc', '2': 'Huyền', '3': 'Hỏi', '4': 'Ngã', '5': 'Nặng' };
        logs.push(`[uinput / S2] Nhận phím '${char}': Áp dụng dấu thanh [${toneNames[char]}] lên từ đang gõ`);
        continue;
      }

      // Cancel tone (0)
      if (char === '0') {
        tone = null;
        currentState = 'S1';
        logs.push(`[uinput / S1] Nhận phím '0': Xóa toàn bộ dấu thanh đang áp dụng`);
        continue;
      }

      // Standard alphabetical letters
      resultWord += char;
      currentState = 'S1';
    }

    // Apply the tone accent if any
    if (tone) {
      // Find the correct vowel to put the tone on (Vietnamese grammar rule)
      // Standard rules: usually the last vowel if double vowel, or the middle one.
      // E.g., "hoàng" -> tone on "à", "tiếng" -> tone on "ế"
      // Let's implement a simple heuristic:
      const vowels = ['a', 'e', 'i', 'o', 'u', 'y', 'â', 'ê', 'ô', 'ơ', 'ư', 'ă', 'Ă', 'Â', 'Ê', 'Ô', 'Ơ', 'Ư'];
      let vowelPositions: number[] = [];
      for (let i = 0; i < resultWord.length; i++) {
        if (vowels.includes(resultWord[i].toLowerCase())) {
          vowelPositions.push(i);
        }
      }

      if (vowelPositions.length > 0) {
        // Decide which vowel receives the tone
        let targetIndex = vowelPositions[0];
        if (vowelPositions.length === 2) {
          // If there is "uy", tone is on "y", else "oa", "oe", "ue", "uy", etc.
          const pair = (resultWord[vowelPositions[0]] + resultWord[vowelPositions[1]]).toLowerCase();
          if (pair === 'oa' || pair === 'oe' || pair === 'uâ' || pair === 'uy' || pair === 'iê' || pair === 'yê' || pair === 'uô' || pair === 'ươ') {
            targetIndex = vowelPositions[1];
          } else {
            targetIndex = vowelPositions[0];
          }
        } else if (vowelPositions.length === 3) {
          // Three vowels (e.g. "oai", "uay", "ươu"), tone usually on the middle one
          targetIndex = vowelPositions[1];
        }

        const targetChar = resultWord[targetIndex];
        resultWord = resultWord.substring(0, targetIndex) + targetChar + tone + resultWord.substring(targetIndex + 1);
      }
    }

    processedWords.push(normalizeVietnamese(resultWord));
  }

  // Generate a final state change summary for the diff system
  const finalOutput = processedWords.join(' ');
  if (inputText !== finalOutput && finalOutput !== '') {
    logs.push(`[Token-Level Diffing] Đã đồng bộ sự kiện phím ảo: Thay thế chuỗi "${inputText}" thành "${finalOutput}" trong 1ms`);
  }

  return {
    text: finalOutput,
    logs: logs.length > 0 ? logs : ["Chờ phím gõ từ terminal..."],
    state: currentState,
  };
}
