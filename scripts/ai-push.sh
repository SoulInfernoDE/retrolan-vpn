#!/usr/bin/env bash
# =====================================================================
# RetroLAN AI Auto-Push Tool (Gemini 3.1 Pro Thinking - Bulletproof V3)
# =====================================================================
set -e

if [ -z "$(git status --porcelain)" ]; then
  echo "✔ Working directory clean. Nothing to push."
  exit 0
fi

if [ -z "$GEMINI_API_KEY" ]; then
  echo "❌ Fehler: Die Variable GEMINI_API_KEY ist nicht gesetzt!"
  echo "Bitte führe aus: export GEMINI_API_KEY=\"dein-key\""
  exit 1
fi

echo "🧠 Inspecting changes with Gemini 3.1 Pro (Thinking Engine)..."
DIFF_CONTENT=$(git diff HEAD)

# 1. JSON-Payload sicher mit jq generieren (maskiert automatisch alle Sonderzeichen & Newlines im Diff!)
JSON_PAYLOAD=$(jq -n --arg diff "$DIFF_CONTENT" '{
  contents: [{
    parts: [{
      text: ("You are an expert Rust/Tauri developer analyzing complex git diffs with deep reasoning and extended thinking. Review this git diff and write a concise, conventional commit message (e.g. feat:, fix:, docs:, refactor:) in English. Return ONLY the final commit message string without markdown formatting, code blocks, explanations, or quotes.\n\nDiff:\n" + $diff)
    }]
  }]
}')

# 2. API-Aufruf mit sicherem Payload
API_RESPONSE=$(curl -s -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro-latest:generateContent?key=${GEMINI_API_KEY}" \
  -H 'Content-Type: application/json' \
  -d "$JSON_PAYLOAD")

# 3. Commit-Message extrahieren
COMMIT_MSG=$(echo "$API_RESPONSE" | jq -r '.candidates[0].content.parts[0].text // empty')

# Prüfen, ob die Nachricht leer oder "null" ist (API-Fehler)
if [ -z "$COMMIT_MSG" ] || [ "$COMMIT_MSG" == "null" ]; then
  echo "❌ Gemini API Fehler! Das Modell konnte keine Nachricht generieren."
  echo "Originale API-Antwort zur Diagnose:"
  echo "$API_RESPONSE" | jq .
  exit 1
fi

echo "✔ Gemini generierte Nachricht: \"$COMMIT_MSG\""

git add .
git commit -m "$COMMIT_MSG"

echo "🚀 Pushing to GitHub..."
if git push origin main; then
  echo "✨ Successfully pushed to GitHub: \"$COMMIT_MSG\""
else
  echo "❌ Git Push ist fehlgeschlagen! Bitte prüfe deine Token-Berechtigungen."
  exit 1
fi
