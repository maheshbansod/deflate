import { constants, deflateRaw } from "node:zlib";

if (import.meta.main) {
  const originalText =
    `Some text. Oh text.. oh text oh text ooh.. yayyyy ayayayya.
    I am a text fan aaaahhh.. I"m a text fan. I need just text. Gimme the text! Gimme.`;
  deflateRaw(
    originalText,
    { level: constants.Z_NO_COMPRESSION },
    async (e, r) => {
      if (e) {
        return;
      }
      console.log("boutta wrte");
      try {
        await Deno.writeFile("test_data/nocompression-deflate2.deflate", r)
          .catch(
            console.log,
          );
        console.log("test");
      } catch (e) {
        console.log(e);
      }
    },
  );
}
