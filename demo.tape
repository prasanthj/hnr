Output demo.gif

Set Shell "bash"
Set FontSize 15
Set Width 1600
Set Height 900
Set Theme "Catppuccin Mocha"
Set Padding 24
Set PlaybackSpeed 1
Set TypingSpeed 80ms

# Show install command, then launch
Type "cargo install hnr"
Sleep 1s
Enter
Sleep 18s

# Clear and launch
Type "hnr"
Enter
Sleep 5s

# Browse top stories
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 600ms

# Open comments
Enter
Sleep 3500ms

# Read through comments
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

# Full text overlay
Enter
Sleep 2s

# Scroll the overlay
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

# Close overlay, go back
Escape
Sleep 500ms
Escape
Sleep 800ms

# Search
Type "?"
Sleep 700ms
Type "rust async"
Sleep 500ms
Enter
Sleep 4s

# Browse results
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

# Bookmark a result
Type "b"
Sleep 1000ms

# Switch to bookmarks
Type "7"
Sleep 2s

# Quit
Type "q"
Sleep 500ms
