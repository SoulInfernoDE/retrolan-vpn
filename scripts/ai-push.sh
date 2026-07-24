#!/usr/bin/env bash
# =====================================================================
# RetroLAN AI Auto-Push Tool (Gemini Powered)
# =====================================================================
if [ -z "$(git status --porcelain)" ]; then
  echo "✔ Working directory clean. Nothing to push."
  exit 0
fi

echo "🧠 Inspecting changes with Gemini..."
DIFF_CONTENT=$(git diff HEAD)
COMMIT_MSG=$(curl -s -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=${GEMINI_API_KEY}" \
  -H 'Content-Type: application/json' \
  -d '{
    "contents": [{
      "parts": [{
        "text": "You are an expert Rust/Tauri developer. Review this git diff and write a concise, conventional commit message (e.g. feat:, fix:, docs:) in English. Return ONLY the commit message string without markdown or quotes.\n\nDiff:\n'"${DIFF_CONTENT}"'"
      }]
    }]
  }' | jq -r '.candidates[0].content.parts[0].text')

git add .
git commit -m "$COMMIT_MSG"
git push origin main
echo "🚀 Successfully pushed to GitHub: \"$COMMIT_MSG\""
