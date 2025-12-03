@0xde0c83658ae6cafd;

struct TfsFile {
  name  @0 :Text;
  inode @1 :UInt64;
  owner @2 :UInt32;
  group @3 :UInt32;
  permissions  @4 :UInt16;
  whenAccessed @5 :UInt64;
  whenModified @6 :UInt64;
  whenChanged  @7 :UInt64;
  tags         @8 :List(UInt64);
}
