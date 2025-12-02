@0x8174264e7a7d34e2;

struct TagFilesystem {
  files      @0 :List(TfsFile);
  tags       @1 :List(TfsTag);
  namespaces @2 :List(TfsNamespace);
}

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

struct TfsTag {
  name  @0 :Text;
  inode @1 :UInt64;
  owner @2 :UInt32;
  group @3 :UInt32;
  permissions  @4 :UInt16;
  whenAccessed @5 :UInt64;
  whenModified @6 :UInt64;
  whenChanged  @7 :UInt64;
}

struct TfsNamespace {
  name  @0 :Text;
  inode @1 :UInt64;
  tags  @2 :List(UInt64);
}
