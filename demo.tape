Output demo.gif

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

# Launch
Type "hnr"
Enter
Sleep 4s

# Quickly scroll through a few Top stories
Type "j"
Sleep 150ms
Type "j"
Sleep 150ms
Type "j"
Sleep 150ms
Type "j"
Sleep 150ms
Type "j"
Sleep 150ms
Type "j"
Sleep 200ms

# Switch to Best feed
Type "3"
Sleep 4s

# Move to second article
Type "j"
Sleep 500ms

# Open reader
Type "r"
Sleep 10s

# Scroll line by line
Type "j"
Sleep 250ms
Type "j"
Sleep 250ms
Type "j"
Sleep 250ms
Type "j"
Sleep 250ms
Type "j"
Sleep 250ms

# Section jumps
Type "n"
Sleep 1200ms
Type "n"
Sleep 1200ms

# Page down
Type "d"
Sleep 900ms
Type "d"
Sleep 900ms

# Jump to end then back to top
Type "G"
Sleep 1000ms
Type "g"
Sleep 900ms

# Close reader
Escape
Sleep 700ms

# Fetch comments
Enter
Sleep 4500ms

# Navigate comment tree
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Expand inline
Enter
Sleep 1200ms

# Collapse thread
Type " "
Sleep 900ms

# Expand back
Type " "
Sleep 700ms

# Upvote
Type "v"
Sleep 1000ms

# Yank URL
Type "y"
Sleep 800ms

# Back to stories pane
Tab
Sleep 500ms

# Algolia search
Type "?"
Sleep 500ms
Type "ratatui"
Sleep 400ms
Enter
Sleep 4s

# Navigate to 4th result
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Sleep 1800ms

# Bookmark 4th result
Type "b"
Sleep 2000ms

# Switch to bookmark feed
Type "7"
Sleep 3000ms

# Quit
Type "q"
Sleep 500ms
