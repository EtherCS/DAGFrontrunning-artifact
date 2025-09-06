use std::collections::BTreeSet;

// Copyright(C) Facebook, Inc. and its affiliates.
// #[cfg(feature = "benchmark")]
use crate::attacks::{
    Attacker, ATTACKING_INTERVAL, FISSURE_ATTACK, FISSURE_NOT_ACTIVE_ATTACK, HONEST_NODE,
    MONITOR_NOTHING, SPECULATIVE_ATTACK, SPECULATIVE_NOT_DELEGATE_ATTACK, SLUGGISH_ATTACK,
    SLUGGISH_DELAY_MULTI_FACTOR, SLUGGISH_NOT_ACTIVE_ATTACK,
};
use crate::messages::{Certificate, Header};
use crate::primary::Round;
use config::{Committee, WorkerId};
use crypto::Hash as _;
use crypto::{Digest, PublicKey, SignatureService};
#[cfg(feature = "benchmark")]
use log::info;
use log::{debug, log_enabled, warn};
use std::cmp::Ordering;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{sleep, Duration, Instant};

#[cfg(test)]
#[path = "tests/proposer_tests.rs"]
pub mod proposer_tests;

/// The proposer creates new headers and send them to the core for broadcasting and further processing.
pub struct Proposer {
    /// The public key of this primary.
    name: PublicKey,
    /// The committee information.
    committee: Committee,
    /// Service to sign headers.
    signature_service: SignatureService,
    /// The size of the headers' payload.
    header_size: usize,
    /// The maximum delay to wait for batches' digests.
    max_header_delay: u64,

    /// Receives the parents to include in the next header (along with their round number).
    rx_core: Receiver<(Vec<Certificate>, Round)>,
    /// Receives the batches' digests from our workers.
    rx_workers: Receiver<(Digest, WorkerId)>,
    /// Sends newly created headers to the `Core`.
    tx_core: Sender<Header>,
    /// Receives ordered certificate from garbage collector to decide if stop creating propagation priority blocks
    rx_clean_certificate: Receiver<Certificate>,
    /// Receives a victim certificate from core, start attacking this victim certificate
    rx_monitor_header: Receiver<Header>,

    /// The current round of the dag.
    round: Round,
    /// Holds the certificates' ids waiting to be included in the next header.
    last_parents: Vec<Certificate>,
    /// Holds the certificate of the last leader (if any).
    last_leader: Option<Certificate>,
    /// Holds the batches' digests waiting to be included in the next header.
    digests: Vec<(Digest, WorkerId)>,
    /// Keeps track of the size (in bytes) of batches' digests that we received so far.
    payload_size: usize,

    /// The parameters of attacker
    attacker: Attacker,

    /// Trace the current victim header (block)
    /// 1) victim_header.author == self.name: no victim header currently
    /// 2) victim_header.author != self.name: we are front-running
    victim_header: Header,
    is_attacking: bool,
}

impl Proposer {
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        name: PublicKey,
        committee: Committee,
        signature_service: SignatureService,
        header_size: usize,
        max_header_delay: u64,
        rx_core: Receiver<(Vec<Certificate>, Round)>,
        rx_workers: Receiver<(Digest, WorkerId)>,
        tx_core: Sender<Header>,
        rx_clean_certificate: Receiver<Certificate>,
        rx_monitor_header: Receiver<Header>,
        attacker: Attacker,
    ) {
        let genesis = Certificate::genesis(&committee);
        tokio::spawn(async move {
            let mut ini_victim_header = Header::default();
            ini_victim_header.author = name;
            Self {
                name,
                committee,
                signature_service,
                header_size,
                max_header_delay,
                rx_core,
                rx_workers,
                tx_core,
                rx_clean_certificate,
                rx_monitor_header,
                round: 0,
                last_parents: genesis,
                last_leader: None,
                digests: Vec::with_capacity(2 * header_size),
                payload_size: 0,
                attacker: attacker,
                victim_header: ini_victim_header,
                is_attacking: false,
            }
            .run()
            .await;
        });
    }

    async fn make_header(&mut self) {
        // Make a new header.
        let header = Header::new(
            self.name,
            self.round,
            self.digests.drain(..).collect(),
            self.last_parents.drain(..).map(|x| x.digest()).collect(),
            &mut self.signature_service,
        )
        .await;
        debug!("Created {:?}", header);

        #[cfg(feature = "benchmark")]
        {
            for digest in header.payload.keys() {
                // NOTE: This log entry is used to compute performance.
                info!("Created {} -> {:?}", header, digest);
            }
            if header.round % ATTACKING_INTERVAL == 0 && self.attacker.attack_type() == HONEST_NODE
            {
                // Info: Create a victim header
                info!(
                    "FairDag Created a victim header {} in round {}",
                    header.id, header.round
                );
            }
            // Print the attacking information for calculation
            // Note that we only consider a block as a front-running block if and only if: The block is created after the victim block
            // For Fissure and Rushed attacks, we only consider the victim and attacking blocks are in the same round

            // For comparison between adopting attack strategies and not adopting attack strategies
            if header.round % ATTACKING_INTERVAL == 0
                && self.attacker.attack_type() == MONITOR_NOTHING
                && self.victim_header.round == header.round
            {
                info!(
                    "FairDag Monitor attacking header {}, victim header {}, in round {}",
                    header.id, self.victim_header.id, header.round
                );
            }

            if header.round % ATTACKING_INTERVAL == 0
                && self.attacker.attack_type() == FISSURE_ATTACK
                && self.victim_header.round == header.round
            {
                info!(
                    "FairDag Created a fissure attacking header {}, victim header {}, in round {}",
                    header.id, self.victim_header.id, header.round
                );
            }

            if self.attacker.attack_type() == SLUGGISH_ATTACK
                && self.victim_header.round >= header.round
            {
                info!(
                    "FairDag Created a sluggish attacking header {}, victim header {}, in round {}",
                    header.id, self.victim_header.id, header.round
                );
            }
        }

        // Send the new header to the `Core` that will broadcast and process it.
        self.tx_core
            .send(header)
            .await
            .expect("Failed to send header");
    }

    // pick-minumum attack: create a header with the minimum id (i.e., the header's digest)
    // return the number of left payload's digests
    async fn make_minimum_header(&mut self) -> usize {
        // ensure there is at least one digest (otherwise, we have no choices)
        // and only consider the attacking round
        let mut left_payload = 0;
        if !self.digests.is_empty()
            && self.round % ATTACKING_INTERVAL == 0
            && self.victim_header.round == self.round
        {
            let temp_last_parents: BTreeSet<Digest> =
                self.last_parents.drain(..).map(|x| x.digest()).collect();
            let temp_payload_digests = self.digests.clone();
            let mut min_index = 0;

            let first_header_id = self.attacker.get_header_id(
                self.name,
                self.round,
                min_index,
                &temp_payload_digests,
                &temp_last_parents,
            );
            let mut min_digest =
                self.attacker
                    .get_certificate_digest(self.name, self.round, &first_header_id);

            #[cfg(feature = "benchmark")]
            {
                info!("Current digest amount is {}", temp_payload_digests.len());
            }
            // now try to pick a minimum digest
            for element_num in 1..self
                .attacker
                .speculative_try_times()
                .min(temp_payload_digests.len())
            {
                // calculate a new digest
                let try_header_id = self.attacker.get_header_id(
                    self.name,
                    self.round,
                    element_num,
                    &temp_payload_digests,
                    &temp_last_parents,
                );
                let temp_digest =
                    self.attacker
                        .get_certificate_digest(self.name, self.round, &try_header_id);
                if temp_digest > min_digest {
                    // reverted tree traversal
                    // therefore, pick the maximum
                    min_digest = temp_digest;
                    min_index = element_num;
                }
            }
            // calculate the left payload
            left_payload = self.digests.len() - min_index - 1;
            // left_payload = 0;
            // Make a new header.
            let header = Header::new(
                self.name,
                self.round,
                self.digests.drain(0..(min_index + 1)).collect(),
                temp_last_parents,
                &mut self.signature_service,
            )
            .await;
            debug!("Created {:?}", header);

            #[cfg(feature = "benchmark")]
            {
                for digest in header.payload.keys() {
                    // NOTE: This log entry is used to compute performance.
                    info!("Created {} -> {:?}", header, digest);
                }
                if self.attacker.attack_type() == SPECULATIVE_ATTACK {
                    // Info: Create a speculative attacking header
                    info!(
                        "FairDag Created a speculative attacking header {}, victim header {}, in round {}",
                        header.id, self.victim_header.id, header.round
                    );
                }
            }

            // Send the new header to the `Core` that will broadcast and process it.
            self.tx_core
                .send(header)
                .await
                .expect("Failed to send header");
        } else {
            // Make a new header.
            let header = Header::new(
                self.name,
                self.round,
                self.digests.drain(..).collect(),
                self.last_parents.drain(..).map(|x| x.digest()).collect(),
                &mut self.signature_service,
            )
            .await;
            debug!("Created {:?}", header);

            #[cfg(feature = "benchmark")]
            {
                for digest in header.payload.keys() {
                    // NOTE: This log entry is used to compute performance.
                    info!("Created {} -> {:?}", header, digest);
                }

                if header.round % ATTACKING_INTERVAL == 0
                    // && self.attacker.attack_type() == SPECULATIVE_ATTACK
                    && self.victim_header.round == header.round
                // this is the target victim block
                {
                    // Info: Create a speculative attacking header
                    info!(
                        "FairDag Created a speculative attacking header {}, victim header {}, in round {}",
                        header.id, self.victim_header.id, header.round
                    );
                }
            }

            // Send the new header to the `Core` that will broadcast and process it.
            self.tx_core
                .send(header)
                .await
                .expect("Failed to send header");
        }
        left_payload
    }

    /// Update the last leader.
    fn update_leader(&mut self) -> bool {
        let leader_name = self.committee.leader(self.round as usize);
        self.last_leader = self
            .last_parents
            .iter()
            .find(|x| x.origin() == leader_name)
            .cloned();

        if let Some(leader) = self.last_leader.as_ref() {
            debug!("Got leader {} for round {}", leader.origin(), self.round);
        }

        self.last_leader.is_some()
    }

    /// Check whether if we have (i) 2f+1 votes for the leader, (ii) f+1 nodes not voting for the leader,
    /// or (iii) there is no leader to vote for.
    fn enough_votes(&self) -> bool {
        let leader = match &self.last_leader {
            Some(x) => x.digest(),
            None => return true,
        };

        let mut votes_for_leader = 0;
        let mut no_votes = 0;
        for certificate in &self.last_parents {
            let stake = self.committee.stake(&certificate.origin());
            if certificate.header.parents.contains(&leader) {
                votes_for_leader += stake;
            } else {
                no_votes += stake;
            }
        }

        let mut enough_votes = votes_for_leader >= self.committee.quorum_threshold();
        if log_enabled!(log::Level::Debug) && enough_votes {
            if let Some(leader) = self.last_leader.as_ref() {
                debug!(
                    "Got enough support for leader {} at round {}",
                    leader.origin(),
                    self.round
                );
            }
        }
        enough_votes |= no_votes >= self.committee.validity_threshold();
        enough_votes
    }

    /// Main loop listening to incoming messages.
    pub async fn run(&mut self) {
        debug!("Dag starting at round {}", self.round);
        let mut advance = true;

        let timer = sleep(Duration::from_millis(self.max_header_delay));
        tokio::pin!(timer);

        loop {
            // Check if we can propose a new header. We propose a new header when we have a quorum of parents
            // and one of the following conditions is met:
            // (i) the timer expired (we timed out on the leader or gave up gather votes for the leader),
            // (ii) we have enough digests (minimum header size) and we are on the happy path (we can vote for
            // the leader or the leader has enough votes to enable a commit).
            let enough_parents = !self.last_parents.is_empty();
            let enough_digests = self.payload_size >= self.header_size;
            let timer_expired = timer.is_elapsed();
            if self.attacker.attack_type() == HONEST_NODE
                || self.attacker.attack_type() == MONITOR_NOTHING
            {
                if (timer_expired || (enough_digests && advance)) && enough_parents {
                    if timer_expired {
                        warn!("Timer expired for round {}", self.round);
                    }

                    // Advance to the next round.
                    self.round += 1;
                    debug!("Dag moved to round {}", self.round);

                    // Make a new header.
                    self.make_header().await;
                    self.payload_size = 0;

                    // Reschedule the timer.
                    let deadline = Instant::now() + Duration::from_millis(self.max_header_delay);
                    timer.as_mut().reset(deadline);
                }
            } else if self.attacker.attack_type() == FISSURE_ATTACK
                || self.attacker.attack_type() == FISSURE_NOT_ACTIVE_ATTACK
            {
                // Current: in core.rs, do not add victim block into header.parents
                if (timer_expired || (enough_digests && advance)) && enough_parents {
                    if timer_expired {
                        warn!("Timer expired for round {}", self.round);
                    }

                    // Advance to the next round.
                    self.round += 1;
                    debug!("Dag moved to round {}", self.round);

                    // Make a new header.
                    self.make_header().await;
                    self.payload_size = 0;

                    // Reschedule the timer.
                    let deadline = Instant::now() + Duration::from_millis(self.max_header_delay);
                    timer.as_mut().reset(deadline);
                }
            } else if self.attacker.attack_type() == SLUGGISH_ATTACK
                || self.attacker.attack_type() == SLUGGISH_NOT_ACTIVE_ATTACK
            {
                // Current: in core.rs, do not add victim block into header.parents
                if (timer_expired || (enough_digests && advance)) && enough_parents {
                    if timer_expired {
                        warn!("Timer expired for round {}", self.round);
                    }

                    // Advance to the next round.
                    self.round += 1;
                    debug!("Dag moved to round {}", self.round);

                    // if (timer_expired) && enough_parents {
                    // Make a new header.
                    self.make_header().await;
                    self.payload_size = 0;

                    if self.is_attacking {
                        // if is attacking, create new blocks in a normal speed
                        let deadline =
                            Instant::now() + Duration::from_millis(self.max_header_delay);
                        timer.as_mut().reset(deadline);
                    } else {
                        // otherwise, slow down the creation of new blocks
                        // current setting: double max_header_delay
                        let deadline = Instant::now()
                            + Duration::from_millis(
                                SLUGGISH_DELAY_MULTI_FACTOR * self.max_header_delay,
                            );
                        timer.as_mut().reset(deadline);
                    }
                }
            } else if self.attacker.attack_type() == SPECULATIVE_ATTACK
                || self.attacker.attack_type() == SPECULATIVE_NOT_DELEGATE_ATTACK
            {
                // currently, we only consider one node in pick-minimum attack
                if (timer_expired || (enough_digests && advance))
                    && enough_parents
                    && self.is_attacking
                {
                    if timer_expired {
                        warn!("Timer expired for round {}", self.round);
                    }

                    // Advance to the next round.
                    self.round += 1;
                    debug!("Dag moved to round {}", self.round);

                    self.payload_size = self.make_minimum_header().await;

                    // Reschedule the timer.
                    let deadline = Instant::now() + Duration::from_millis(self.max_header_delay);
                    timer.as_mut().reset(deadline);
                } else if (timer_expired || (enough_digests && advance)) && enough_parents {
                    if timer_expired {
                        warn!("Timer expired for round {}", self.round);
                    }

                    // Advance to the next round.
                    self.round += 1;
                    debug!("Dag moved to round {}", self.round);

                    // Make a new header.
                    self.make_header().await;
                    self.payload_size = 0;

                    // Reschedule the timer.
                    let deadline = Instant::now() + Duration::from_millis(self.max_header_delay);
                    timer.as_mut().reset(deadline);
                }
            }

            tokio::select! {
                Some((parents, round)) = self.rx_core.recv() => {
                    // Compare the parents' round number with our current round.
                    match round.cmp(&self.round) {
                        Ordering::Greater => {
                            // We accept round bigger than our current round to jump ahead in case we were
                            // late (or just joined the network).
                            self.round = round;
                            self.last_parents = parents;
                        },
                        Ordering::Less => {
                            // Ignore parents from older rounds.
                        },
                        Ordering::Equal => {
                            // The core gives us the parents the first time they are enough to form a quorum.
                            // Then it keeps giving us all the extra parents.
                            self.last_parents.extend(parents)
                        }
                    }

                    // Check whether we can advance to the next round. Note that if we timeout,
                    // we ignore this check and advance anyway.
                    advance = match self.round % 2 {
                        0 => self.update_leader(),
                        _ => self.enough_votes(),
                    }
                }
                Some((digest, worker_id)) = self.rx_workers.recv() => {
                    self.payload_size += digest.size();
                    self.digests.push((digest, worker_id));
                }
                Some(header) = self.rx_monitor_header.recv() => {
                    // Find a victim header, start front-running it by constructing propagation priority blocks
                    if self.attacker.attack_type() == FISSURE_ATTACK || self.attacker.attack_type() == FISSURE_NOT_ACTIVE_ATTACK {
                        if !self.is_attacking && self.round <= header.round {
                            self.victim_header.update(header);
                            self.is_attacking = true;
                        }
                    } else if self.attacker.attack_type() == SLUGGISH_ATTACK || self.attacker.attack_type() == SLUGGISH_NOT_ACTIVE_ATTACK {
                        if !self.is_attacking && self.round <= header.round {
                            self.victim_header.update(header);
                            self.is_attacking = true;
                        }
                    } else if (self.attacker.attack_type() == SPECULATIVE_ATTACK || self.attacker.attack_type() == SPECULATIVE_NOT_DELEGATE_ATTACK)
                        && !self.is_attacking && self.round <= header.round
                    {
                        self.victim_header.update(header);
                        self.is_attacking = true;
                    } else if self.attacker.attack_type() == MONITOR_NOTHING {
                        if !self.is_attacking && self.round <= header.round {
                            self.victim_header.update(header);
                            self.is_attacking = true;
                        }
                    }
                }
                Some(certificate) = self.rx_clean_certificate.recv() => {
                    if self.attacker.attack_type() == FISSURE_ATTACK || self.attacker.attack_type() == FISSURE_NOT_ACTIVE_ATTACK {
                        if certificate.header.round >= self.victim_header.round {
                            // the victim certificate has been committed or missed, move to attack another certificate
                            self.is_attacking = false;
                        }
                    } else if self.attacker.attack_type() == SLUGGISH_ATTACK || self.attacker.attack_type() == SLUGGISH_NOT_ACTIVE_ATTACK {
                        if certificate.header.round >= self.victim_header.round {
                            // the victim certificate has been committed or missed, move to attack another certificate
                            self.is_attacking = false;
                        }
                    } else if self.attacker.attack_type() == SPECULATIVE_ATTACK || self.attacker.attack_type() == SPECULATIVE_NOT_DELEGATE_ATTACK {
                        if certificate.header.round >= self.victim_header.round {
                            // the victim certificate has been committed or missed, move to attack another certificate
                            self.is_attacking = false;
                        }
                    } else if self.attacker.attack_type() == MONITOR_NOTHING {
                        if certificate.header.round >= self.victim_header.round {
                            // the victim certificate has been committed or missed, move to attack another certificate
                            self.is_attacking = false;
                        }
                    }
                }
                () = &mut timer => {
                    // Nothing to do.
                }
            }
        }
    }
}
