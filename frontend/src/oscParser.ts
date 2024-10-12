export function parseOSCMessage(arrayBuffer: ArrayBuffer) {
  const dataView = new DataView(arrayBuffer);
  let offset = 0;

  // Parse OSC address pattern
  let address = "";
  while (true) {
    const char = dataView.getUint8(offset);
    if (char === 0) break;
    address += String.fromCharCode(char);
    offset++;
  }
  offset = (offset + 4) & ~3; // align to 4-byte boundary

  // Parse type tag string (optional, starts with a comma)
  let typeTags = "";
  if (dataView.getUint8(offset) === 44) {
    // ASCII for ','
    offset++;
    while (true) {
      const char = dataView.getUint8(offset);
      if (char === 0) break;
      typeTags += String.fromCharCode(char);
      offset++;
    }
    offset = (offset + 4) & ~3; // align to 4-byte boundary
  }

  // Parse arguments based on type tags
  const args = [];
  for (let i = 0; i < typeTags.length; i++) {
    switch (typeTags[i]) {
      case "i": // Integer
        args.push(dataView.getInt32(offset));
        offset += 4;
        break;
      case "f": // Float
        args.push(dataView.getFloat32(offset));
        offset += 4;
        break;
      case "s": // String
        let str = "";
        while (true) {
          const char = dataView.getUint8(offset);
          if (char === 0) break;
          str += String.fromCharCode(char);
          offset++;
        }
        offset = (offset + 4) & ~3; // align to 4-byte boundary
        args.push(str);
        break;
      case "b": // Blob (binary data)
        const size = dataView.getInt32(offset);
        offset += 4;
        const blob = arrayBuffer.slice(offset, offset + size);
        args.push(blob);
        offset += size;
        offset = (offset + 4) & ~3; // align to 4-byte boundary
        break;
      case "h": // int64 (Unix timestamp)
        const high = dataView.getInt32(offset);
        const low = dataView.getInt32(offset + 4);
        const int64 = high * 2 ** 32 + low;
        args.push(int64);
        offset += 8;
        break;
      default:
        throw new Error("Unsupported OSC type tag: " + typeTags[i]);
    }
  }

  return { address, args };
}
