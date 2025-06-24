# Discrete Event Simulation Models Repository

This repository contains multiple discrete event simulation models implemented using the [nexosim](https://docs.rs/nexosim/latest/nexosim/) framework in Rust.

## Projects

### MultiplierExample
A simple example demonstrating the basic usage of the nexosim simulation framework. This model shows how to create custom components, connect them via ports, and collect statistics.

[View MultiplierExample details](./MultiplierExample/README.md)

### SimPoolSim
A simulation of a swimming pool with capacity constraints. Models visitor arrivals, time spent in the pool, and waiting queue behavior when the pool reaches maximum capacity.

[View SwimPoolSim details](SwimPoolSim/README.md)

## Running the Simulations

Each project can be run independently. Navigate to the specific project directory and follow the instructions in its README.

### Requirements
- Rust and Cargo installed (https://www.rust-lang.org/tools/install)

## Repository Structure

- `SimPoolSim/` - Swimming pool capacity simulation
- `MultiplierExample/` - Basic nexosim framework usage example

## Contributing

To contribute a new simulation model:
1. Create a new directory for your model
2. Implement your simulation using the nexosim framework
3. Include a detailed README with model description and usage instructions
4. Submit a pull request