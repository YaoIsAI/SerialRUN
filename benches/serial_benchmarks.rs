use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serialrun_core::protocol::modbus::*;
use serialrun_core::protocol::custom::*;

fn bench_modbus_crc(c: &mut Criterion) {
    let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
    c.bench_function("modbus_crc16", |b| {
        b.iter(|| ModbusFrame::calculate_crc(black_box(&data)))
    });
}

fn bench_modbus_frame_parse(c: &mut Criterion) {
    let frame = ModbusFrame::new(0x01, ModbusFunction::ReadHoldingRegisters, vec![0x00, 0x00, 0x00, 0x0A]);
    let bytes = frame.to_bytes();
    c.bench_function("modbus_frame_parse", |b| {
        b.iter(|| ModbusFrame::parse(black_box(&bytes)))
    });
}

fn bench_modbus_frame_build(c: &mut Criterion) {
    c.bench_function("modbus_frame_build", |b| {
        b.iter(|| {
            ModbusFrame::new(
                black_box(0x01),
                black_box(ModbusFunction::ReadHoldingRegisters),
                black_box(vec![0x00, 0x00, 0x00, 0x0A]),
            )
        })
    });
}

fn bench_protocol_parser(c: &mut Criterion) {
    let parser = ProtocolParser::default();
    let data = b"AT+RST\r\n";
    c.bench_function("protocol_parser_at", |b| {
        b.iter(|| parser.parse(black_box(data)))
    });
}

fn bench_protocol_parser_json(c: &mut Criterion) {
    let parser = ProtocolParser::default();
    let data = b"{\"key\": \"value\"}";
    c.bench_function("protocol_parser_json", |b| {
        b.iter(|| parser.parse(black_box(data)))
    });
}

criterion_group!(
    benches,
    bench_modbus_crc,
    bench_modbus_frame_parse,
    bench_modbus_frame_build,
    bench_protocol_parser,
    bench_protocol_parser_json,
);
criterion_main!(benches);
