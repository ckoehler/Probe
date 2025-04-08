use tokio_stream::StreamExt;

use crossterm::event::KeyEvent;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::error;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wraps keyboard input and tick events. Each event
/// type is handled in its own task and returned to a common `Receiver`
#[derive(Debug)]
pub struct Events {
    rx: mpsc::Receiver<Event<KeyEvent>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel(10);
        {
            let tx = tx.clone();
            let mut reader = crossterm::event::EventStream::new();
            tokio::spawn(async move {
                loop {
                    if let Ok(crossterm::event::Event::Key(key)) =
                        reader.next().await.expect("Failed to read terminal event")
                    {
                        if let Err(err) = tx.send(Event::Input(key)).await {
                            error!("{}", err);
                        }
                    }
                }
            });
        }
        {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(config.tick_rate);
                loop {
                    if tx.send(Event::Tick).await.is_err() {
                        break;
                    }
                    interval.tick().await;
                }
            })
        };
        Events { rx }
    }

    pub async fn next(&mut self) -> Option<Event<KeyEvent>> {
        self.rx.recv().await
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_events_config() {
        let config = Config {
            tick_rate: Duration::from_millis(100),
        };
        let events = Events::with_config(config);
        assert!(events.rx.capacity() > 0);
    }

    #[tokio::test]
    async fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.tick_rate, Duration::from_millis(250));
    }

    #[tokio::test]
    async fn test_events_tick_generation() {
        let config = Config {
            tick_rate: Duration::from_millis(100),
        };
        let mut events = Events::with_config(config);

        // We should receive a tick event within a reasonable timeframe
        let result = timeout(Duration::from_millis(200), events.next()).await;
        assert!(result.is_ok(), "Timed out waiting for tick event");

        if let Ok(Some(Event::Tick)) = result {
            // This is the expected case
        } else {
            panic!("Expected tick event, got {:?}", result.err());
        }
    }

    #[tokio::test]
    async fn test_events_multiple_ticks() {
        let config = Config {
            tick_rate: Duration::from_millis(50),
        };
        let mut events = Events::with_config(config);

        // We should receive multiple tick events
        let mut tick_count = 0;
        for _ in 0..3 {
            if let Some(Event::Tick) = timeout(Duration::from_millis(100), events.next())
                .await
                .unwrap()
            {
                tick_count += 1;
            }
        }

        assert!(
            tick_count >= 2,
            "Expected at least 2 ticks, got {tick_count}"
        );
    }
}
