use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use tokio::{
  sync::{mpsc, oneshot, Mutex},
  task::JoinHandle,
};

use crate::{
  components::{home::Home, Component},
  terminal::{EventHandler, TuiHandler},
  trace_dbg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
  Quit,
  Tick,
  Resize(u16, u16),
  ToggleShowLogger,
  ScheduleIncrementCounter,
  ScheduleDecrementCounter,
  AddToCounter(usize),
  SubtractFromCounter(usize),
  EnterNormal,
  EnterInsert,
  EnterProcessing,
  ExitProcessing,
  Update,
  Noop,
}

pub struct App {
  pub tick_rate: u64,
  pub home: Arc<Mutex<Home>>,
}

impl App {
  pub fn new(tick_rate: u64) -> Result<Self> {
    let home = Arc::new(Mutex::new(Home::new()));
    Ok(Self { tick_rate, home })
  }

  pub fn spawn_tui_task(&mut self) -> (JoinHandle<()>, oneshot::Sender<()>) {
    let home = self.home.clone();

    let (stop_tui_tx, mut stop_tui_rx) = oneshot::channel::<()>();

    let tui_task = tokio::spawn(async move {
      let mut tui = TuiHandler::new().context(anyhow!("Unable to create TUI")).unwrap();
      tui.enter().unwrap();
      loop {
        let mut h = home.lock().await;
        tui
          .terminal
          .draw(|f| {
            h.render(f, f.size());
          })
          .unwrap();
        if stop_tui_rx.try_recv().ok().is_some() {
          break;
        }
      }
      tui.exit().unwrap();
    });

    (tui_task, stop_tui_tx)
  }

  pub fn spawn_event_task(&mut self, tx: mpsc::UnboundedSender<Action>) -> (JoinHandle<()>, oneshot::Sender<()>) {
    let home = self.home.clone();
    let tick_rate = self.tick_rate;
    let (stop_event_tx, mut stop_event_rx) = oneshot::channel::<()>();
    let event_task = tokio::spawn(async move {
      let mut events = EventHandler::new(tick_rate);
      loop {
        // get the next event
        let event = events.next().await;

        // map event to an action
        let action = home.lock().await.handle_events(event);

        // add action to action handler channel queue
        tx.send(action).unwrap();

        if stop_event_rx.try_recv().ok().is_some() {
          events.stop().await.unwrap();
          break;
        }
      }
    });
    (event_task, stop_event_tx)
  }

  pub async fn run(&mut self) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    self.home.lock().await.tx = Some(tx.clone());

    self.home.lock().await.init()?;

    let (tui_task, stop_tui_tx) = self.spawn_tui_task();
    let (event_task, stop_event_tx) = self.spawn_event_task(tx.clone());

    loop {
      // clear all actions from action handler channel queue
      let mut maybe_action = rx.try_recv().ok();
      while maybe_action.is_some() {
        let action = maybe_action.unwrap();
        if action != Action::Tick {
          trace_dbg!(action);
        }
        if let Some(action) = self.home.lock().await.dispatch(action) {
          tx.send(action)?
        };
        maybe_action = rx.try_recv().ok();
      }

      // quit state
      if self.home.lock().await.should_quit {
        stop_tui_tx.send(()).unwrap_or(());
        stop_event_tx.send(()).unwrap_or(());
        tui_task.await?;
        event_task.await?;
        break;
      }
    }
    Ok(())
  }
}
