Output preview.gif

Set Shell "bash"
Set FontSize 14
Set Width 1400
Set Height 800
Set Theme "Catppuccin Mocha"
Set Padding 0
Set PlaybackSpeed 0.9
Set TypingSpeed 80ms
Set WaitTimeout 12s

# Launch
Type "/Users/prasanth/work/fun/hnr/target/debug/hnr"
Enter
Sleep 4s

# Browse top stories
Sleep 1s
Down
Sleep 400ms
Down
Sleep 400ms
Down
Sleep 600ms

# Open comments
Enter
Sleep 3s

# Scroll comments
Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 500ms

# Expand a comment
Enter
Sleep 800ms

# Collapse it
Space
Sleep 600ms

# Back to stories
Escape
Sleep 500ms

# Switch to New feed
Type "2"
Sleep 2.5s

# Browse a couple
Down
Sleep 400ms
Down
Sleep 700ms

# Switch to Ask HN
Type "4"
Sleep 2.5s

# Browse
Down
Sleep 400ms
Down
Sleep 600ms

# Back to Top
Type "1"
Sleep 2s

# Quit
Type "q"
Sleep 500ms
