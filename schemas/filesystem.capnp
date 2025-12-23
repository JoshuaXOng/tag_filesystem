@0x8174264e7a7d34e2;

using import "root.capnp".RootFuser;
using import "file.capnp".TfsFile;
using import "tag.capnp".TfsTag;

struct TagFilesystem {
  root       @2 :RootFuser;
  files      @0 :List(TfsFile);
  tags       @1 :List(TfsTag);
}
