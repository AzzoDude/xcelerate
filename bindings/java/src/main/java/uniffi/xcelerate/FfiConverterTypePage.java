package uniffi.xcelerate;


public enum FfiConverterTypePage implements FfiConverter<Page, java.lang.Long> {
    INSTANCE;

    @Override
    public java.lang.Long lower(Page value) {
        return value.uniffiCloneHandle();
    }

    @Override
    public Page lift(java.lang.Long value) {
        return new Page(UniffiWithHandle.INSTANCE, value);
    }

    @Override
    public Page read(java.nio.ByteBuffer buf) {
        // The Rust code always writes handles as 8 bytes, and will
        // fail to compile if they don't fit.
        return lift(buf.getLong());
    }

    @Override
    public long allocationSize(Page value) {
        return 8L;
    }

    @Override
    public void write(Page value, java.nio.ByteBuffer buf) {
        // The Rust code always expects handles written as 8 bytes,
        // and will fail to compile if they don't fit.
        buf.putLong(lower(value));
    }
}



