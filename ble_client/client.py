import asyncio
import sys
import termios
import tty
from bleak import BleakScanner, BleakClient, BleakError

# Name: BT24, Address/UUID: 7DD7B727-DD9D-C402-8CE6-4F8C843D472B
ADDRESS_OR_UUID = "7DD7B727-DD9D-C402-8CE6-4F8C843D472B"
TX_CHAR_UUID = "0000ffe1-0000-1000-8000-00805f9b34fb" 
RX_CHAR_UUID = "0000ffe1-0000-1000-8000-00805f9b34fb"

def read_single_char() -> bytes:
    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    try:
        tty.setcbreak(fd)
        ch = sys.stdin.read(1)
        return ch.encode("utf-8")
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
async def keyboard_char_writer(client: BleakClient, char_uuid: str):
    loop = asyncio.get_running_loop()

    while client.is_connected:
        try:
            char_bytes = await loop.run_in_executor(None, read_single_char)

            # Check for Exit shortcuts (Ctrl+C: \x03, Ctrl+D: \x04)
            if char_bytes in (b"\x03", b"\x04"):
                break

            # Transmit character immediately
            # Note: response=False (Write Without Response) is often faster for streaming
            await client.write_gatt_char(char_uuid, char_bytes, response=False)

            # Echo locally to stdout
            display_char = "\\r" if char_bytes == b"\r" else char_bytes.decode('utf-8', errors='replace')
            print(f"[TX Char] '{display_char}' ({char_bytes.hex()})")

        except Exception as e:
            print(f"Error sending char: {e}\r\n")
            break
def handle_notification(sender, data: bytearray):
    r = data.decode('ascii', errors='replace')
    if "[INFO] [->]" in r:
        print(f"{r[7:]}")
        
    else: 
        print(f"[<-] {data.decode('ascii', errors='replace')}")

async def main():
    while True:
        print(f"Attempting to connect to {ADDRESS_OR_UUID}...")
        try:
            async with BleakClient(ADDRESS_OR_UUID) as client:
                print(f"Connected: {client.is_connected}")
                print(f"Subscribing to RX: {RX_CHAR_UUID}")
                await client.start_notify(RX_CHAR_UUID, handle_notification)
                print(f"Subscribing to TX: {TX_CHAR_UUID}")
                await keyboard_char_writer(client, TX_CHAR_UUID)

                while client.is_connected:
                    await asyncio.sleep(1)

        except BleakError as e:
            print(f"Connection error: {e}")
        except Exception as e:
            print(f"Unexpected error: {e}")

        print("Disconnected. Reconnecting in 3 seconds...\n")
        await asyncio.sleep(3)

asyncio.run(main())

async def scanner():
    devices = await BleakScanner.discover()
    for d in devices:
        print(f"Name: {d.name}, Address/UUID: {d.address}")

