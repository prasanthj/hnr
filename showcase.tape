Output showcase.gif

Set Shell "bash"
Set FontSize 15
Set Width 1600
Set Height 900
Set Theme "Catppuccin Mocha"
Set Padding 0
Set PlaybackSpeed 3.0
Set TypingSpeed 80ms
Set WaitTimeout 30s
Set CursorBlink false

Hide
Type "export PATH=$HOME/.cargo/bin:$PATH"
Enter
Sleep 200ms
Type "rm -f ~/.hnr/seen.json"
Enter
Sleep 200ms
Show

# Install
Type "cargo install hnr"
Enter
Sleep 4s

# Launch
Type "hnr"
Enter
Sleep 4s

# Refresh feed
Type "R"
Sleep 3s

# Navigate up and down
Down
Sleep 600ms
Down
Sleep 600ms
Down
Sleep 600ms
Up
Sleep 500ms
Up
Sleep 700ms

# Switch to Best
Type "3"
Sleep 3s

# Navigate to story 3 for reader
Down
Sleep 600ms
Down
Sleep 700ms

# Open reader
Type "r"
Sleep 4s

# Navigate paragraphs with n
Type "n"
Sleep 800ms
Type "n"
Sleep 800ms
Type "n"
Sleep 800ms
Type "n"
Sleep 800ms
Type "N"
Sleep 800ms
Type "N"
Sleep 1000ms

# Close reader
Type "q"
Sleep 700ms

# Back to story 1, load comments
Up
Sleep 400ms
Up
Sleep 600ms
Enter
Sleep 3500ms

# Navigate comments
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 600ms

# Expand a comment
Enter
Sleep 1000ms

# Collapse it
Space
Sleep 800ms

# Back to stories
Escape
Sleep 600ms

# Search for ratatui
Type "?"
Sleep 500ms
Type "ratatui"
Sleep 400ms
Enter
Sleep 3s

# Navigate results and bookmark one
Down
Sleep 500ms
Down
Sleep 700ms
Type "b"
Sleep 600ms

# Switch to Bookmarks
Type "7"
Sleep 3500ms

# Quit
Type "q"
Sleep 500ms
