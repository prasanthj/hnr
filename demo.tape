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

# Switch to Best feed — longer, meatier articles
Type "3"
Sleep 4s

# Story 1 on Best
Type "r"
Sleep 12s

# Line scroll to show content moving
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Jump between sections
Type "n"
Sleep 1400ms
Type "n"
Sleep 1400ms

# Half-page down
Type "d"
Sleep 1000ms
Type "d"
Sleep 1000ms

# Jump to end
Type "G"
Sleep 1200ms

# Back to top
Type "g"
Sleep 1000ms

# Close reader
Escape
Sleep 700ms

# Fetch threaded comments
Enter
Sleep 4500ms

# Navigate comment tree
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Collapse a thread
Type " "
Sleep 800ms
Type " "
Sleep 600ms

# View author profile — press p
Type "p"
Sleep 2500ms
Escape
Sleep 700ms

# Switch back to stories
Tab
Sleep 600ms

# Bookmark the story
Type "b"
Sleep 700ms

# Browse the bookmark feed
Type "7"
Sleep 1500ms

# Back to top feed
Type "1"
Sleep 900ms

# Full-text search via Algolia
Type "?"
Sleep 500ms
Type "ratatui"
Sleep 400ms
Enter
Sleep 4s

# Browse results — pause so viewer reads them
Type "j"
Sleep 350ms
Type "j"
Sleep 350ms
Sleep 2000ms

# Bookmark this search result — pause to show star appear
Type "b"
Sleep 2000ms

# Show bookmark feed — pause to show the saved story
Type "7"
Sleep 3000ms

# Back to top feed
Type "1"
Sleep 900ms

# Login modal — shows hnr supports voting and replies
Type "l"
Sleep 1800ms
Escape
Sleep 500ms

# Quit
Type "q"
Sleep 500ms
