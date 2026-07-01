use crate::network::NetworkManager;
use crate::AbyssError;
use libp2p::Multiaddr;

pub struct BootstrapManager;

impl BootstrapManager {
    pub async fn bootstrap(network: &mut NetworkManager, bootstrap_nodes: &[String]) -> Result<(), AbyssError> {
        for node in bootstrap_nodes {
            let multiaddr: Multiaddr = node.parse()
                .map_err(|e| AbyssError::Network(format!("Invalid addr: {}", e)))?;
            let peer_id = multiaddr.iter().last()
                .and_then(|p| match p {
                    libp2p::multiaddr::Protocol::P2p(hash) => Some(libp2p::PeerId::from_multihash(hash).unwrap()),
                    _ => None,
                })
                .ok_or(AbyssError::Network("No PeerId".to_string()))?;
            network.swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
        }
        network.swarm.behaviour_mut().kademlia.bootstrap()
            .map_err(|e| AbyssError::Network(format!("Bootstrap: {}", e)))?;
        Ok(())
    }
}