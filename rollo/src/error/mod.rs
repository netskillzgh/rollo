#[derive(Debug, Clone)]
pub enum Error {
    PacketSize,
    NumberConversion,
    ReadingPacket,
    DosProtection,
    TimeoutReading,
    Channel,
    PacketPayload,
    TlsAcceptTimeout,
}
