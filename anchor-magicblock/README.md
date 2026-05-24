# aeroscan protocol

A decentralized air quality monitoring system built on Solana with Ephemeral Rollups integration for efficient sensor data management.

## Overview

Aeroscan is a blockchain-based protocol that enables secure, verifiable storage and management of environmental sensor data. The protocol tracks air quality metrics including PM2.5, PM10 levels, temperature, and humidity readings while leveraging Ephemeral Rollups for cost-effective and high-throughput data processing.

## Features

- **Real-time Air Quality Monitoring**: Track PM2.5, PM10, temperature, and humidity
- **Ephemeral Rollups Integration**: Efficient data processing with delegation capabilities
- **Decentralized Storage**: Immutable sensor data records on Solana
- **Event Emission**: Real-time notifications for sensor reading updates
- **Authority-based Access Control**: Secure sensor data management

## Architecture

The protocol consists of four main operations:

### Core Instructions

1. **Initialize**: Create a new sensor reading account for a user
2. **Update Reading**: Submit new sensor data readings
3. **Delegate**: Move sensor data processing to Ephemeral Rollups for efficiency
4. **Undelegate**: Return sensor data from Ephemeral Rollups back to base layer

### Data Structure

Each sensor reading contains:
- **Authority**: The public key of the sensor owner
- **PM2.5**: Particulate matter 2.5 microns (µg/m³)
- **PM10**: Particulate matter 10 microns (µg/m³)
- **Temperature**: Temperature reading (unit depends on implementation)
- **Humidity**: Relative humidity percentage
- **Timestamp**: Unix timestamp of the reading

## Getting Started

### Prerequisites

- Node.js (v16 or higher)
- Solana CLI tools
- Anchor framework
- TypeScript

### Installation

1. Install dependencies:
```bash
pnpm install
```

1. Build the program:
```bash
anchor build
```

1. Deploy to devnet:
```bash
yarn deploy:devnet
```

### Environment Setup

The protocol is configured to work with:
- **RPC Endpoint**: Helius Devnet
- **Ephemeral Rollups**: MagicBlock Devnet
- **Network**: Solana Devnet

## Usage

### Initialize Sensor Reading Account

```typescript
await program.methods
  .initialize(pm25, pm10, temperature, humidity, aqi)
  .accountsPartial({
    sensorReading: sensor_reading_pda,
    user: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### Update Sensor Readings

```typescript
await program.methods
  .updateReading(authority, pm25, pm10, temperature, humidity, aqi)
  .accountsPartial({
    sensorReading: sensor_reading_pda,
  })
  .rpc();
```

### Delegate to Ephemeral Rollups

```typescript
await program.methods
  .delegate()
  .accounts({
    payer: wallet.publicKey,
    sensorReading: sensor_reading_pda,
  })
  .rpc();
```

### Undelegate from Ephemeral Rollups

```typescript
await program.methods
  .undelegate()
  .accountsPartial({
    user: wallet.publicKey,
    sensorReading: sensor_reading_pda,
  })
  .rpc();
```

## Program Addresses

- **Program ID**: `aero8wSmn3uAj5g5jYq92Rd2SQv2MtGxu1ZXfysfFHX`
- **Sensor Reading PDA**: Derived from `["sensor_reading", user_pubkey]`

## Events

The protocol emits `SensorReadingEvent` whenever sensor data is updated, containing:
- PM2.5 and PM10 levels
- Temperature and humidity readings
- Timestamp of the measurement

## Testing

Run the test suite:

```bash
anchor test
```

The tests cover:
1. Account initialization
2. Base layer sensor data updates
3. Delegation to Ephemeral Rollups
4. Ephemeral Rollups sensor data updates
5. Undelegation back to base layer

## Use Cases

- **Environmental Monitoring Networks**: Deploy sensors across cities or regions
- **Industrial Compliance**: Track air quality for regulatory compliance
- **Research Projects**: Collect verifiable environmental data for studies
- **IoT Integration**: Connect physical sensors to blockchain infrastructure
- **Data Markets**: Create marketplaces for verified environmental data

## Technical Benefits

### Ephemeral Rollups Integration
- **Cost Efficiency**: Reduce transaction costs for frequent sensor updates
- **High Throughput**: Process multiple readings without base layer congestion
- **Flexible Processing**: Delegate data to rollups when needed, undelegate when required

### Blockchain Advantages
- **Immutable Records**: Tamper-proof environmental data
- **Verifiable Data**: Cryptographically secured sensor readings
- **Decentralized**: No single point of failure or control
- **Transparent**: Public access to environmental data

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## Security Considerations

- Sensor readings are protected by authority-based access control
- Only the sensor owner can update their readings
- All transactions are signed and verified on-chain
- Ephemeral Rollups provide additional security layers

## License

[LICENSE](LICENSE)

## Acknowledgments

- [Solana](https://solana.com/)
- [MagicBlock](https://magicblock.xyz/)
- [Helius](https://helius.com/)
- [Anchor](https://anchor-lang.com/)