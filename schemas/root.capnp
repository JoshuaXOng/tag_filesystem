@0xcc21e33910c1ca98;

struct RootFuser {
  owner         @0 :UInt32;
  group         @1 :UInt32;
  permissions   @2 :UInt16;
  whenAccessedSeconds       @3 :UInt64;
  whenAccessedNanoseconds   @4 :UInt32;
  whenModifiedSeconds       @5 :UInt64;
  whenModifiedNanoseconds   @6 :UInt32;
  whenChangedSeconds        @7 :UInt64;
  whenChangedNanoseconds    @8 :UInt32;
  whenCreatedSeconds        @9 :UInt64;
  whenCreatedNanoseconds    @10 :UInt32;
}
