# The game

Player is given a list of words and must reconstruct a board of letters
that allows spelling out all the words, Word Hunt style. Depending on game
configuration, they might get some free letters already placed into the
board, or a list of letters theyneed to insert.

# Project Structure

## Frontend

Vite + React

Components for:
- Board
- Word list

## Backend

Rust

- main.rs: entry point of the backend
- api.rs: provides all the APIs called by the frontend
- words.rs: defines a Trie for storing all the words, provides functionality for searching for words
- board.rs: provides functionality for generating a game off a given config