# Technology & Systems Catalog

Curated technical observations, empirical trade-offs, and engineering theses for the `tech-curator` skill.

---

## 1. Embedded SQLite vs. Distributed Microservice Sprawl
- **The Thesis**: 95% of software products never exceed 100 queries per second, yet their architectures look like Netflix in 2018.
- **The Reality**: Modern PCI-e 4.0/5.0 NVMe drives sustain $>7,000\text{ MB/s}$ read speeds and hundreds of thousands of random IOPS. A single SQLite instance in Write-Ahead Logging (WAL) mode embedded directly inside a compiled Rust or Go process avoids network serialization, TCP handshakes, TLS termination, and inter-service authentication tokens.
- **The Tradeoff**: Distributed databases are mandatory for multi-region active-active writes; for everything else, they primarily generate cloud vendor bills.

---

## 2. Sensor Area Physics vs. Computational Photography
- **The Thesis**: Software algorithms can denoise, tone-map, and synthetic-aperture blur, but they cannot manufacture photons that never hit the silicon.
- **The Reality**: A $35\text{mm}$ full-frame sensor has an area of $864\text{ mm}^2$, compared to $\approx 60\text{ mm}^2$ on a modern flagship smartphone ($14.4\times$ light gathering surface). The signal-to-noise ratio (SNR) is governed by Poisson photon shot noise $\sqrt{N}$. When illumination drops, computational photography inevitably invents high-frequency texture through hallucination or over-smooths subtle gradations.
- **The Tradeoff**: The best camera is the one you have with you; but physics remains undefeated in low-light dynamic range.

---

## 3. Local Quantized SLMs (3B–8B) vs. Cloud Mega-LLMs
- **The Thesis**: Using a 400B parameter cloud API to parse a JSON payload or extract three entities is computational absurdity.
- **The Reality**: Modern 4-bit (AWQ / GGUF) quantized 3B–8B models (e.g. Llama-3.2, Qwen-2.5) run on consumer Apple Silicon unified memory or a modest laptop GPU at $>60\text{ tokens/sec}$ with zero network latency, zero cloud egress costs, and absolute data privacy.
- **The Tradeoff**: Complex multi-step frontier reasoning still demands massive parameter scale, but structured operational tasks belong on the edge.

---

## 4. Memory Safety & Cache Locality
- **The Thesis**: High throughput in modern CPUs is primarily memory-bound, not ALU-bound.
- **The Reality**: L1 cache latency is $\approx 1\text{ ns}$ (4 cycles); main RAM access is $50\text{--}80\text{ ns}$ (200+ cycles). Garbage-collected languages with object pointer indirection (AoS) cause the CPU prefetcher to stall constantly on cache misses. Struct-of-Arrays (SoA) memory layouts and zero-cost compile-time ownership tracking (Rust borrow checker) maximize cache line utilization and eliminate stop-the-world latency spikes.
- **The Tradeoff**: Developer cognitive load during compilation vs runtime predictability and memory efficiency.

---

## 5. Hall-Effect Magnetic Switches vs. Mechanical Leaves
- **The Thesis**: Physical copper leaf contacts in keyboards are fundamentally prone to oxidation, chatter, and fixed physical actuation points.
- **The Reality**: Hall-effect sensors measure the continuous magnetic flux of a rare-earth magnet attached to the key stem. This unlocks rapid trigger (resetting actuation the instant the key ascends by $0.1\text{ mm}$), dynamic multi-stage actuations, and zero contact bouncing.
- **The Tradeoff**: Analog sensor power draw and calibration tolerances vs superior physical responsiveness.
