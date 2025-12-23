@0xde0c83658ae6cafd;

struct TfsFile {
  name          @0 :Text;
  inode         @1 :UInt64;
  owner         @2 :UInt32;
  group         @3 :UInt32;
  permissions   @4 :UInt16;
  whenAccessedSeconds       @5 :UInt64;
  whenAccessedNanoseconds   @6 :UInt32;
  whenModifiedSeconds       @7 :UInt64;
  whenModifiedNanoseconds   @8 :UInt32;
  whenChangedSeconds        @9 :UInt64;
  whenChangedNanoseconds    @10 :UInt32;
  whenCreatedSeconds        @11 :UInt64;
  whenCreatedNanoseconds    @12 :UInt32;
  tags          @13 :List(UInt64);
}
