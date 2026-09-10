use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct KademliaRoutingTable {
    known_peers: HashSet<String>,
}

impl KademliaRoutingTable {
    pub fn new() -> Self {
        Self {
            known_peers: HashSet::new(),
        }
    }

    pub fn add_peer(&mut self, peer_id: String) -> bool {
        self.known_peers.insert(peer_id)
    }

    pub fn peer_count(&self) -> usize {
        self.known_peers.len()
    }
}