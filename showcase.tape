Output showcase.gif

Set Shell "zsh"
Set FontSize 15
Set Width 1600
Set Height 900
Set Theme "Catppuccin Mocha"
Set Padding 24
Set PlaybackSpeed 2
Set TypingSpeed 80ms

Hide
Type "export PATH=$HOME/.cargo/bin:$PATH"
Enter
Sleep 200ms
Type "cp /Users/prasanth/work/fun/hnr/target/release/hnr ~/.cargo/bin/hnr"
Enter
Sleep 500ms
Show

# Launch — Refreshing shimmer visible while stories load
Type "hnr"
Enter
Sleep 4500ms

# Scroll quickly through a few stories
Type "j"
Sleep 150ms
Type "j"
Sleep 150ms
Type "j"
Sleep 200ms

# Settle — shimmer restarts, 3s fires reader prefetch
Sleep 3000ms

# Open reader — Fetching article shimmer
Type "r"
Sleep 6s

# Navigate within reader
Type "j"
Sleep 250ms
Type "j"
Sleep 250ms
Type "n"
Sleep 900ms
Type "n"
Sleep 900ms
Type "d"
Sleep 700ms
Type "G"
Sleep 700ms
Type "g"
Sleep 500ms

# Close reader — comments already prefetched
Escape
Sleep 400ms

# Enter comments — instant
Enter
Sleep 900ms

# Navigate comment tree
Type "j"
Sleep 250ms
Type "j"
Sleep 250ms
Enter
Sleep 700ms
Type " "
Sleep 500ms

# Vote and yank
Type "v"
Sleep 700ms
Type "y"
Sleep 500ms

# Back to stories
Tab
Sleep 400ms

# Algolia search
Type "?"
Sleep 400ms
Type "ratatui"
Sleep 300ms
Enter
Sleep 3500ms

# Navigate to 3rd result — dwell for comment prefetch shimmer
Type "j"
Sleep 250ms
Type "j"
Sleep 250ms
Sleep 5500ms

# Bookmark after comments arrive
Type "b"
Sleep 800ms

# Switch to bookmarks feed and quit
Type "7"
Sleep 2000ms
Type "q"
Sleep 300ms
