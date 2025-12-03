@0x8174264e7a7d34e2;

using import "file.capnp".TfsFile;
using import "tag.capnp".TfsTag;

struct TagFilesystem {
  files      @0 :List(TfsFile);
  tags       @1 :List(TfsTag);
}
