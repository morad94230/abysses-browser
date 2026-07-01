use crate::AbyssError;
use libp2p::{
    identity::Keypair as Libp2pKeypair,
    kad::{store::MemoryStore, Kademlia, KademliaConfig},
    mdns::tokio::Behaviour as MdnsBehaviour,
    noise,
    quic::tokio::Transport as QuicTransport,
    swarm::SwarmBuilder,
    PeerId as Libp2pPeerId, Swarm,
};
use std::time::Duration;

pub mod bootstrap;

pub struct NetworkManager {
    pub swarm: Swarm<AbyssBehaviour>,
    pub local_peer_id: Libp2pPeerId,
    pub keypair: Libp2pKeypair,
}

#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct AbyssBehaviour {
    pub kademlia: Kademlia<MemoryStore>,
    pub mdns: MdnsBehaviour,
    pub identify: libp2p::identify::Behaviour,
}

impl NetworkManager {
    pub async fn new(
        keypair: Libp2pKeypair,
        listen_addresses: Vec<String>,
    ) -> Result<Self, AbyssError> {
        let peer_id = Libp2pPeerId::from(keypair.public());
        let quic_transport = QuicTransport::new(libp2p::quic::Config::default(), &keypair)
            .map_err(|e| AbyssError::Network(format!("QUIC: {}", e)))?;
        let noise_config = noise::Config::new_xx(&keypair)
            .map_err(|e| AbyssError::Network(format!("Noise: {}", e)))?;
        let noise_transport =
            noise::NoiseAuthenticated::new(quic_transport, noise_config, &keypair)
                .map_err(|e| AbyssError::Network(format!("Noise: {}", e)))?;
        let behaviour = AbyssBehaviour {
            kademlia: Kademlia::with_config(
                peer_id,
                MemoryStore::new(peer_id),
                KademliaConfig::default().set_query_timeout(Duration::from_secs(30)),
            ),
            mdns: MdnsBehaviour::new(libp2p::mdns::Config::default(), peer_id)
                .map_err(|e| AbyssError::Network(format!("mDNS: {}", e)))?,
            identify: libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                "/abyss/1.0.0".to_string(),
                keypair.public(),
            )),
        };
        let swarm = SwarmBuilder::with_tokio_executor(noise_transport, behaviour, peer_id).build();
        let mut nm = Self {
            swarm,
            local_peer_id: peer_id,
            keypair,
        };
        for addr in listen_addresses {
            nm.swarm
                .listen_on(
                    addr.parse()
                        .map_err(|e| AbyssError::Network(format!("Addr: {}", e)))?,
                )
                .map_err(|e| AbyssError::Network(format!("Listen: {}", e)))?;
        }
        Ok(nm)
    }

    pub fn local_peer_id_string(&self) -> String {
        self.local_peer_id.to_string()
    }
    pub fn noise_public_key(&self) -> Vec<u8> {
        self.keypair.public().encode_protobuf()
    }
}
