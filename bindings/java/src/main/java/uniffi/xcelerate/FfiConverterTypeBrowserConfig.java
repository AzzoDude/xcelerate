package uniffi.xcelerate;


public enum FfiConverterTypeBrowserConfig implements FfiConverterRustBuffer<BrowserConfig> {
  INSTANCE;

  @Override
  public BrowserConfig read(java.nio.ByteBuffer buf) {
    return new BrowserConfig(
      FfiConverterBoolean.INSTANCE.read(buf),
      FfiConverterBoolean.INSTANCE.read(buf),
      FfiConverterBoolean.INSTANCE.read(buf),
      FfiConverterOptionalString.INSTANCE.read(buf),
      FfiConverterOptionalSequenceString.INSTANCE.read(buf)
    );
  }

  @Override
  public long allocationSize(BrowserConfig value) {
      return (
            FfiConverterBoolean.INSTANCE.allocationSize(value.headless()) +
            FfiConverterBoolean.INSTANCE.allocationSize(value.stealth()) +
            FfiConverterBoolean.INSTANCE.allocationSize(value.detached()) +
            FfiConverterOptionalString.INSTANCE.allocationSize(value.executablePath()) +
            FfiConverterOptionalSequenceString.INSTANCE.allocationSize(value.plugins())
      );
  }

  @Override
  public void write(BrowserConfig value, java.nio.ByteBuffer buf) {
      FfiConverterBoolean.INSTANCE.write(value.headless(), buf);
      FfiConverterBoolean.INSTANCE.write(value.stealth(), buf);
      FfiConverterBoolean.INSTANCE.write(value.detached(), buf);
      FfiConverterOptionalString.INSTANCE.write(value.executablePath(), buf);
      FfiConverterOptionalSequenceString.INSTANCE.write(value.plugins(), buf);
  }
}


