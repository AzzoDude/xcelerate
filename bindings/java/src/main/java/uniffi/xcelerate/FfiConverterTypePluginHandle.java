package uniffi.xcelerate;


public enum FfiConverterTypePluginHandle implements FfiConverter<PluginHandle, java.lang.Long> {
    INSTANCE;

    @Override
    public java.lang.Long lower(PluginHandle value) {
        return value.uniffiCloneHandle();
    }

    @Override
    public PluginHandle lift(java.lang.Long value) {
        return new PluginHandle(UniffiWithHandle.INSTANCE, value);
    }

    @Override
    public PluginHandle read(java.nio.ByteBuffer buf) {
        // The Rust code always writes handles as 8 bytes, and will
        // fail to compile if they don't fit.
        return lift(buf.getLong());
    }

    @Override
    public long allocationSize(PluginHandle value) {
        return 8L;
    }

    @Override
    public void write(PluginHandle value, java.nio.ByteBuffer buf) {
        // The Rust code always expects handles written as 8 bytes,
        // and will fail to compile if they don't fit.
        buf.putLong(lower(value));
    }
}



