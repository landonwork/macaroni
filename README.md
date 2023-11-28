# Macaroni
## Making Rust macros easy

Macaroni was inspired by my desire to learn to write my own Rust proc-macros, get a taste for writing a GUI
application, and make something other Rustaceans might like. In my time learning and designing Macaroni, I
tried Dioxus, egui, and Iced, and even looked at web frameworks and Tauri, exploring all of my
options for creating a user-friendly interface. Writing the Rust macros themselves and expanding them is
quite easy. Making something intuitive and presentable is much more difficult.

I eventually settled on Iced because the state management and
sharing is straight-forward, I like the view/update design and message-passing paradigm, and recently support
was added for a text editor with syntax highlighting.

## Features

The bar for this project is embarrassingly low. I am a first-time GUI developer, and I have only one goal
for Macaroni which is to have the application set up a Cargo project with the correct template and
compile and expand the macros as I am working on them.

- [x] Write an attribute macro, compile, and expand it
- [ ] Write an declarative macro, and expand it
- [ ] Write an derive macro, compile, and expand it
- [ ] Write an function-like macro, compile, and expand it
- [ ] Insert spaces when tab is pressed
- [ ] Choose a theme where underscores are visible
- [ ] Expand macros in real-time, without having to click on a button
- [ ] See Cargo.toml
- [ ] Edit Cargo.toml

## Development

I'm working on this on my own time. I welcome conversation with anyone who wants to learn more about Rust
and Rust macros. I'll take suggestions, but this is a small project, not really a huge open-source endeavor.
Would I like for this to be good enough that people want to use it? Absolutely, which is why I will consider
your input. You are also more than welcome to fork this repository and develop your own flavor of Macaroni.

## Use

Until I figure out what bundler I can use with Iced to make Macaroni into a proper desktop app, your best bet is
to compile from source on your machine.
