extern crate i3ipc;

use i3ipc::{event::Event, I3Connection, I3EventListener, Subscription};
use linked_hash_set::LinkedHashSet;

const ENHANCE_KEYWORD: &str = "enhanced";

struct Program {
  connection: I3Connection,
  workspaces: LinkedHashSet<String>,
  workspace_tab_stack: Option<Vec<String>>,
}

impl Program {
  pub fn new() -> Self {
    let mut connection = I3Connection::connect().unwrap();
    let workspaces = Self::get_workspaces(&mut connection);
    Program {
      connection,
      workspaces,
      workspace_tab_stack: None,
    }
  }

  fn get_workspaces(connection: &mut I3Connection) -> LinkedHashSet<String> {
    // gets existing workspaces and tries to order them
    let workspaces_full = connection.get_workspaces().unwrap().workspaces;
    let active = workspaces_full
      .iter()
      .find(|wp| wp.focused)
      .unwrap()
      .name
      .clone();
    let mut workspaces: LinkedHashSet<String> =
      workspaces_full.into_iter().map(|wp| wp.name).collect();
    workspaces.insert(active);
    workspaces
  }

  pub fn run(&mut self) {
    let mut listener = I3EventListener::connect().unwrap();

    let subs = [Subscription::Binding];
    listener.subscribe(&subs).unwrap();

    for event in listener.listen() {
      match event.unwrap() {
        Event::BindingEvent(e) => self.handle_binding_event(&e.binding.command),
        _ => unreachable!(),
      }
    }
  }

  fn handle_binding_event(&mut self, s: &str) {
    let words = parse_command_words(s);
    if words[0] == "exec" {
      if words[1] == ENHANCE_KEYWORD {
        if words[2] == "super" {
          if let Some(stack) = &mut self.workspace_tab_stack {
            // update workspaces set and clear workspace stack
            self
              .workspaces
              .insert(self.workspaces.iter().nth(stack.len()).unwrap().clone());
            self.workspace_tab_stack = None;
          }
          return;
        } else if words[2] == "workspace" && words[3] == "tab" {
          if self.workspaces.len() > 1 {
            let tabbed_workspace: String = if let Some(stack) = &mut self.workspace_tab_stack {
              if let Some(popped) = stack.pop() {
                popped
              } else {
                // reset stack and pop the first workspace
                let mut stack: Vec<String> = self.workspaces.iter().map(|s| s.clone()).collect();
                let popped = stack.pop().unwrap();
                self.workspace_tab_stack = Some(stack);
                popped
              }
            } else {
              let mut stack: Vec<String> = self.workspaces.iter().map(|s| s.clone()).collect();
              stack.pop().unwrap(); // pop the current workspace
              let popped = stack.pop().unwrap();
              self.workspace_tab_stack = Some(stack);
              popped
            };
            self
              .connection
              .run_command(&format!("workspace number \"{}\"", tabbed_workspace))
              .unwrap();
          }
        }
      }
    } else if words[0] == "workspace" {
      if words[1] == "number" {
        self.workspaces.insert(words[2].to_string());
      }
    }
  }
}

fn parse_command_words(s: &str) -> Vec<String> {
  // Parses commands
  // Example:
  // parse_command_words("hello \"long sentence\""
  // ->  "hello", "long sentence"

  let mut words = Vec::new();

  let mut cur_word = String::new();
  let mut in_quotes = false;
  for c in s.chars() {
    if in_quotes {
      if c == '"' {
        in_quotes = false;
        words.push(cur_word);
        cur_word = String::new();
      } else {
        cur_word.push(c);
      }
    } else {
      if c == '"' {
        in_quotes = true;
      } else if c == ' ' {
        words.push(cur_word);
        cur_word = String::new();
      } else {
        cur_word.push(c);
      }
    }
  }
  if cur_word.len() > 0 {
    words.push(cur_word);
  }

  words
}

fn main() {
  let mut program = Program::new();
  program.run();
}
