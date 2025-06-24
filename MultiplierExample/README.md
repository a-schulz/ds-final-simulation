# MultiplierExample

This is a working example of a simulation using the [nexosim](https://docs.rs/nexosim/latest/nexosim/) crate. The example demonstrates a simple multiplier component that processes numbers and collects statistics.

## Model structure
```mermaid
flowchart LR
    M1[multiplier1] -->|output -> input| D1[delay1]
    M1 -->|output -> input| M2[multiplier2]
    M2 -->|output -> input| D2[delay2]
    D1 -->|output -> input| D2
    
    %% Monitoring connections
    M1 -.->|output| MS[monitoring_slot]
    M2 -.->|output| MS
    D1 -.->|output| MS
    D2 -->|output| OS[output_slot]
    
    %% Input entry point
    IN[input] -->|21.0| M1
```

## Overview

This example shows how to:
- Set up a basic simulation with nexosim
- Create a custom component (Multiplier)
- Create a custom component with delay (Delay)
- Connect components via ports
- Run the simulation
- Collect and display statistics

## Implementation Details

The example contains a `Multiplier` component that:
- Takes a number input
- Multiplies it by a configurable factor
- Outputs the result

## Running the Example

To run the example:

```bash
cargo run
```

## Dependencies

- nexosim: The simulation framework

## Project Structure

- `src/main.rs`: Main simulation code

## Statistics

The example collects the following statistics:
- Total number of multiplications performed
- Average input value
- Average output value
- Maximum output value

## Additional Resources

For more information, see the [nexosim documentation](https://docs.rs/nexosim/latest/nexosim/).