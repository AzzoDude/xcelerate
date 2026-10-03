package uniffi.xcelerate;


// This is a helper for safely passing byte references into the rust code.
// It's not actually used at the moment, because there aren't many things that you
// can take a direct pointer to in the JVM, and if we're going to copy something
// then we might as well copy it into a `RustBuffer`. But it's here for API
// completeness.
public final class ForeignBytes {
    public static final java.lang.foreign.StructLayout LAYOUT = java.lang.foreign.MemoryLayout.structLayout(
        java.lang.foreign.ValueLayout.JAVA_INT.withName("len"),
        java.lang.foreign.MemoryLayout.paddingLayout(4),  // 4 bytes padding for alignment before ADDRESS
        java.lang.foreign.ValueLayout.ADDRESS.withName("data")
    );

    private static final long OFFSET_LEN = LAYOUT.byteOffset(java.lang.foreign.MemoryLayout.PathElement.groupElement("len"));
    private static final long OFFSET_DATA = LAYOUT.byteOffset(java.lang.foreign.MemoryLayout.PathElement.groupElement("data"));

    private ForeignBytes() {}

    public static int getLen(java.lang.foreign.MemorySegment seg) {
        return seg.get(java.lang.foreign.ValueLayout.JAVA_INT_UNALIGNED, OFFSET_LEN);
    }

    public static void setLen(java.lang.foreign.MemorySegment seg, int value) {
        seg.set(java.lang.foreign.ValueLayout.JAVA_INT_UNALIGNED, OFFSET_LEN, value);
    }

    public static java.lang.foreign.MemorySegment getData(java.lang.foreign.MemorySegment seg) {
        return seg.get(java.lang.foreign.ValueLayout.ADDRESS_UNALIGNED, OFFSET_DATA);
    }

    public static void setData(java.lang.foreign.MemorySegment seg, java.lang.foreign.MemorySegment value) {
        seg.set(java.lang.foreign.ValueLayout.ADDRESS_UNALIGNED, OFFSET_DATA, value);
    }
}
