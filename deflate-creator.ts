import { constants, deflateRaw, createDeflateRaw } from "node:zlib";
import { Buffer } from "node:buffer"

if (import.meta.main) {
  const originalText =
    `Some text. Oh text.. oh text oh text ooh.. yayyyy ayayayya.
    I am a text fan aaaahhh.. I"m a text fan. I need just text. Gimme the text! Gimme.`;
// const originalText = `a a a a a a a a a a`;
  deflateRaw(
    originalText,
    // { strategy: constants.Z_FIXED },
    { level: constants.Z_BEST_COMPRESSION, strategy: constants.Z_FIXED },
    async (e, r) => {
      if (e) {
        return;
      }
	for (const byte of r) {
	    console.log(byte.toString(2).padStart(8, "0"), byte); // Print each byte as binary
	}
      try {
        await Deno.writeFile("test_data/fixed-comp-deflate.deflate", r)
          .catch(
            console.log,
          );
      } catch (e) {
        console.error(e);
      }
    },
  );

	//  const fixedTree = new Map();
	//    // const deflate = createDeflateRaw({ strategy: constants.Z_FIXED });
	//    const deflate = createDeflateRaw({
	// compression: constants.Z_NO_COMPRESSION,
	// strategy: constants.Z_DEFAULT_STRATEGY
	//    });
	//
	//    deflate.on("data", (chunk) => {
	//    });
	//
	//    deflate.end(Buffer.from(originalText));
}
