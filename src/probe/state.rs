// use crate::probe::config;
use regex::Regex;
use std::cmp;
use std::collections::VecDeque;

use super::config::ProbeConfig;

const MESSAGE_BUFFER_SIZE: usize = 60;
const RING_BUFFER_SIZE: usize = 180;

#[derive(Debug)]
pub struct TabsState {
    pub num_tabs: usize,
    pub num_probes: usize,
    pub probes_per_tab: usize,
    pub selected_tab: usize,
    pub selected_probe: usize,
}

impl TabsState {
    pub fn new(num_probes: usize) -> Self {
        Self {
            num_tabs: 1,
            num_probes,
            probes_per_tab: num_probes,
            selected_tab: Default::default(),
            selected_probe: Default::default(),
        }
    }
    pub fn recalculate_layout(&mut self, num_probes: usize, probes_per_tab: usize) {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let new_num_tabs = ((num_probes as f64 / probes_per_tab as f64).ceil()) as usize;

        // if we're changing the layout, likely because of a resize, also reset the currently
        // selected probe and tab.
        if new_num_tabs != self.num_tabs || probes_per_tab != self.probes_per_tab {
            self.selected_probe = 0;
            self.selected_tab = 0;
        }

        self.num_tabs = new_num_tabs;
        self.probes_per_tab = probes_per_tab;
    }

    pub fn next(&mut self) {
        self.selected_probe = 0;
        self.selected_tab = (self.selected_tab + 1) % self.num_tabs;
    }

    pub fn previous(&mut self) {
        if self.selected_tab > 0 {
            self.selected_tab -= 1;
        } else {
            self.selected_tab = self.num_tabs - 1;
        }
        self.selected_probe = 0;
    }

    pub fn next_probe(&mut self) {
        if self.selected_probe < self.probes_on_selected_page().saturating_sub(1) {
            self.selected_probe += 1;
        }
    }

    pub fn prev_probe(&mut self) {
        if self.selected_probe > 0 {
            self.selected_probe -= 1;
        }
    }

    fn probes_on_selected_page(&self) -> usize {
        // if we only have 1 page, return that
        if self.num_probes <= self.probes_per_tab {
            return self.num_probes;
        }

        // all pages but the last
        if self.selected_tab < self.num_tabs - 1 {
            self.probes_per_tab

        // last page
        } else {
            self.num_probes % self.probes_per_tab
        }
    }

    pub fn selected_probe_index(&self) -> usize {
        self.selected_probe + self.selected_tab * self.probes_per_tab
    }
}

#[derive(Debug)]
pub struct AppState {
    pub probes: Vec<Probe>,
    pub detail_view: bool,
}

#[derive(Clone, Debug)]
pub struct Probe {
    pub name: String,
    pub filter: String,
    pub count: u32,
    ring: VecDeque<u64>,
    ring_buffer: u64,
    messages: VecDeque<String>,
}

impl AppState {
    pub fn from_probes(p: &[ProbeConfig]) -> AppState {
        AppState {
            probes: p.iter().map(|i| Probe::from(i.clone())).collect(),
            detail_view: false,
        }
    }

    pub fn probes_for_tab(&self, index: usize, num: usize) -> Vec<Probe> {
        let upper = cmp::min(index * num + num, self.probes.len());
        self.probes[index * num..upper].to_vec()
    }
}

impl Probe {
    pub fn process_message(&mut self, msg: &str) {
        if self.filter.is_empty() {
            self.update_message_buffer(msg);
        } else {
            let re = Regex::new(&self.filter).expect("Failed to parse regex");
            if re.is_match(msg) {
                self.update_message_buffer(msg);
                self.count += 1;
                self.ring_buffer += 1;
            }
        }
    }

    pub fn messages(self) -> String {
        self.messages.clone().make_contiguous().to_vec().join("\n")
    }

    pub fn update_message_buffer(&mut self, msg: &str) {
        self.messages.push_front(msg.to_string());
        if self.messages.len() > MESSAGE_BUFFER_SIZE {
            self.messages.pop_back();
        }
    }

    // this is called once per tick, so do display related stuff here.
    pub fn update_state(&mut self) {
        self.ring.push_front(self.ring_buffer);
        if self.ring.len() >= RING_BUFFER_SIZE {
            self.ring.pop_back();
        }
        self.ring_buffer = 0;
    }

    pub fn histogram(&self) -> Vec<u64> {
        self.ring.clone().make_contiguous().to_vec()
    }
}

impl From<ProbeConfig> for Probe {
    fn from(item: ProbeConfig) -> Self {
        Probe {
            name: item.name,
            filter: item.filter.unwrap_or(".*".to_string()),
            count: 0,
            ring_buffer: 0,
            messages: VecDeque::with_capacity(MESSAGE_BUFFER_SIZE),
            ring: VecDeque::with_capacity(RING_BUFFER_SIZE),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn create_tabstate() {
        let num_probes = 3;
        let probes_per_tab = 1;
        let mut state = TabsState::new(num_probes);
        state.recalculate_layout(num_probes, probes_per_tab);
        assert_eq!(state.selected_probe, 0);
    }

    #[test]
    fn select_next_with_only_one() {
        let num_probes = 3;
        let probes_per_tab = 1;
        let mut state = TabsState::new(num_probes);
        state.recalculate_layout(num_probes, probes_per_tab);
        assert_eq!(state.selected_probe, 0);
        state.next_probe();
        assert_eq!(state.selected_probe, 0);
    }

    #[test]
    fn select_next_with_multiple() {
        let num_probes = 3;
        let probes_per_tab = 2;
        let mut state = TabsState::new(num_probes);
        state.recalculate_layout(num_probes, probes_per_tab);
        assert_eq!(state.selected_probe, 0);
        state.next_probe();
        assert_eq!(state.selected_probe, 1);
        state.next_probe();
        assert_eq!(state.selected_probe, 1);
    }

    #[test]
    fn select_prev_with_only_one() {
        let num_probes = 3;
        let probes_per_tab = 1;
        let mut state = TabsState::new(num_probes);
        state.recalculate_layout(num_probes, probes_per_tab);
        assert_eq!(state.selected_probe, 0);
        state.prev_probe();
        assert_eq!(state.selected_probe, 0);
    }

    #[test]
    fn select_prev_with_multiple() {
        let num_probes = 3;
        let probes_per_tab = 2;
        let mut state = TabsState::new(num_probes);
        state.recalculate_layout(num_probes, probes_per_tab);
        assert_eq!(state.selected_probe, 0);
        state.prev_probe();
        assert_eq!(state.selected_probe, 0);
        state.next_probe();
        assert_eq!(state.selected_probe, 1);
        state.prev_probe();
        assert_eq!(state.selected_probe, 0);
    }

    #[test]
    fn select_next_on_last() {
        let num_probes = 3;
        let probes_per_tab = 2;
        let mut state = TabsState::new(num_probes);
        state.recalculate_layout(num_probes, probes_per_tab);
        state.next_probe();
        assert_eq!(state.selected_probe, 1);

        // next page, where there's only one probe
        state.next();
        assert_eq!(state.selected_probe, 0);

        // should stay at 0, since this page only has one probe
        state.next_probe();
        assert_eq!(state.selected_probe, 0);
    }

    #[test]
    fn probes_on_selected_page_1() {
        let num_probes = 3;
        let probes_per_tab = 2;
        let mut state = TabsState::new(num_probes);
        state.recalculate_layout(num_probes, probes_per_tab);
        assert_eq!(state.probes_on_selected_page(), 2);

        // next page, where there's only one probe
        state.next();
        assert_eq!(state.probes_on_selected_page(), 1);
    }

    // only one page with all the probes
    #[test]
    fn probes_on_selected_page_2() {
        let num_probes = 3;
        let probes_per_tab = 3;
        let mut state = TabsState::new(num_probes);
        state.recalculate_layout(num_probes, probes_per_tab);
        assert_eq!(state.probes_on_selected_page(), 3);
    }

    #[test]
    fn test_selected_probe_index() {
        let num_probes = 3;
        let probes_per_tab = 2;
        let mut state = TabsState::new(num_probes);
        state.recalculate_layout(num_probes, probes_per_tab);
        state.next();
        assert_eq!(state.selected_probe_index(), 2);
    }

    #[test]
    fn test_bug1() {
        let num_probes = 10;
        let probes_per_tab = 8;
        let mut state = TabsState::new(num_probes);
        state.recalculate_layout(num_probes, probes_per_tab);
        assert_eq!(state.probes_on_selected_page(), 8);
        state.next();
        assert_eq!(state.probes_on_selected_page(), 2);
    }

    #[test]
    fn test_appstate_probes_for_tab() {
        // Create some test probe configs
        let configs = vec![
            ProbeConfig {
                name: "probe1".to_string(),
                filter: Some(".*".to_string()),
                address: "localhost:1234".to_string(),
            },
            ProbeConfig {
                name: "probe2".to_string(),
                filter: Some(".*".to_string()),
                address: "localhost:1234".to_string(),
            },
            ProbeConfig {
                name: "probe3".to_string(),
                filter: Some(".*".to_string()),
                address: "localhost:1234".to_string(),
            },
            ProbeConfig {
                name: "probe4".to_string(),
                filter: Some(".*".to_string()),
                address: "localhost:1234".to_string(),
            },
            ProbeConfig {
                name: "probe5".to_string(),
                filter: Some(".*".to_string()),
                address: "localhost:1234".to_string(),
            },
        ];

        let app_state = AppState::from_probes(&configs);

        // Test first tab with 2 probes per tab
        let first_tab = app_state.probes_for_tab(0, 2);
        assert_eq!(first_tab.len(), 2);
        assert_eq!(first_tab[0].name, "probe1");
        assert_eq!(first_tab[1].name, "probe2");

        // Test second tab with 2 probes per tab
        let second_tab = app_state.probes_for_tab(1, 2);
        assert_eq!(second_tab.len(), 2);
        assert_eq!(second_tab[0].name, "probe3");
        assert_eq!(second_tab[1].name, "probe4");

        // Test third tab with 2 probes per tab (partial)
        let third_tab = app_state.probes_for_tab(2, 2);
        assert_eq!(third_tab.len(), 1);
        assert_eq!(third_tab[0].name, "probe5");
    }

    #[test]
    fn test_probe_process_message_with_filter() {
        let mut probe = Probe {
            name: "test_probe".to_string(),
            filter: "error".to_string(),
            count: 0,
            ring_buffer: 0,
            messages: VecDeque::new(),
            ring: VecDeque::new(),
        };

        // Message that matches filter
        probe.process_message("This is an error message");
        assert_eq!(probe.count, 1);
        assert_eq!(probe.ring_buffer, 1);
        assert_eq!(probe.messages.len(), 1);

        // Message that doesn't match filter
        probe.process_message("This is a regular message");
        assert_eq!(probe.count, 1); // should not increase
        assert_eq!(probe.ring_buffer, 1); // should not increase
        assert_eq!(probe.messages.len(), 1); // should not increase
    }

    #[test]
    fn test_probe_process_message_without_filter() {
        let mut probe = Probe {
            name: "test_probe".to_string(),
            filter: String::new(), // Empty filter matches everything
            count: 0,
            ring_buffer: 0,
            messages: VecDeque::new(),
            ring: VecDeque::new(),
        };

        // Empty filter should accept all messages but not increase counters
        probe.process_message("This is a regular message");
        assert_eq!(probe.count, 0); // should not increase with empty filter
        assert_eq!(probe.ring_buffer, 0); // should not increase with empty filter
        assert_eq!(probe.messages.len(), 1); // should still store the message
    }

    #[test]
    fn test_update_message_buffer_capacity() {
        let mut probe = Probe {
            name: "test_probe".to_string(),
            filter: ".*".to_string(),
            count: 0,
            ring_buffer: 0,
            messages: VecDeque::new(),
            ring: VecDeque::new(),
        };

        // Add more than the capacity (MESSAGE_BUFFER_SIZE) messages
        for i in 0..MESSAGE_BUFFER_SIZE + 5 {
            probe.update_message_buffer(&format!("Message {i}"));
        }

        // Should only keep the most recent MESSAGE_BUFFER_SIZE messages
        assert_eq!(probe.messages.len(), MESSAGE_BUFFER_SIZE);
        // The newest message should be at the front
        assert_eq!(probe.messages.front().unwrap(), "Message 64");
        // The oldest kept message should be at the back
        assert_eq!(probe.messages.back().unwrap(), "Message 5");
    }

    #[test]
    fn test_update_state_ring_buffer() {
        let mut probe = Probe {
            name: "test_probe".to_string(),
            filter: ".*".to_string(),
            count: 0,
            ring_buffer: 42, // Set some value to be transferred to ring
            messages: VecDeque::new(),
            ring: VecDeque::new(),
        };

        probe.update_state();
        assert_eq!(probe.ring_buffer, 0); // Should be reset
        assert_eq!(probe.ring.len(), 1);
        assert_eq!(probe.ring.front().unwrap(), &42);

        // Add a few more updates
        probe.ring_buffer = 7;
        probe.update_state();
        probe.ring_buffer = 13;
        probe.update_state();

        assert_eq!(probe.ring.len(), 3);
        let histogram = probe.histogram();
        assert_eq!(histogram, vec![13, 7, 42]);
    }

    #[test]
    fn test_selected_tab_wraparound() {
        let num_probes = 4;
        let probes_per_tab = 2;
        let mut state = TabsState::new(num_probes);
        state.recalculate_layout(num_probes, probes_per_tab);

        // Should have 2 tabs
        assert_eq!(state.num_tabs, 2);

        // Initial state
        assert_eq!(state.selected_tab, 0);

        // Move forward a tab
        state.next();
        assert_eq!(state.selected_tab, 1);

        // Move forward again should wrap around to first tab
        state.next();
        assert_eq!(state.selected_tab, 0);

        // Move backward a tab, should go to last tab
        state.previous();
        assert_eq!(state.selected_tab, 1);
    }

    #[test]
    fn test_recalculate_layout_reset() {
        let mut state = TabsState::new(10);
        state.recalculate_layout(10, 5);

        // Select a different tab and probe
        state.selected_tab = 1;
        state.selected_probe = 1;

        // Recalculate with different layout
        state.recalculate_layout(10, 2);

        // Selection should be reset
        assert_eq!(state.selected_tab, 0);
        assert_eq!(state.selected_probe, 0);

        // But if layout doesn't change, selection should be preserved
        state.selected_tab = 1;
        state.selected_probe = 1;
        state.recalculate_layout(10, 2); // Same parameters as before

        // Selection should not be reset
        assert_eq!(state.selected_tab, 1);
        assert_eq!(state.selected_probe, 1);
    }

    #[test]
    fn test_probe_histogram_empty() {
        let probe = Probe {
            name: "test".to_string(),
            filter: ".*".to_string(),
            count: 0,
            ring_buffer: 0,
            messages: VecDeque::new(),
            ring: VecDeque::new(),
        };

        let hist = probe.histogram();
        assert_eq!(hist.len(), 0);
    }
}
