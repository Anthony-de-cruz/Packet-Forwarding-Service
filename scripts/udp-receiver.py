import socket


LISTEN_HOST = "0.0.0.0"
LISTEN_PORT = 9000


def main() -> None:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind((LISTEN_HOST, LISTEN_PORT))

    print(f"listening on {LISTEN_HOST}:{LISTEN_PORT}")

    packet_count = 0
    byte_count = 0

    try:
        while True:
            data, address = sock.recvfrom(2048)

            packet_count += 1
            byte_count += len(data)
            print(
                f"rx packet {packet_count} from {address[0]}:{address[1]} "
                f"({len(data)} bytes, total {byte_count})"
            )
    except KeyboardInterrupt:
        print(f"\nstopped after {packet_count} packet(s), {byte_count} byte(s)")
    finally:
        sock.close()


if __name__ == "__main__":
    main()
