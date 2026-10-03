package uniffi.xcelerate;


public enum FfiConverterSequenceTypeElement implements FfiConverterRustBuffer<java.util.List<Element>> {
  INSTANCE;

  @Override
  public java.util.List<Element> read(java.nio.ByteBuffer buf) {
    int len = buf.getInt();
    return java.util.stream.IntStream.range(0, len).mapToObj(_i -> FfiConverterTypeElement.INSTANCE.read(buf)).toList();
  }

  @Override
  public long allocationSize(java.util.List<Element> value) {
    long sizeForLength = 4L;
    long sizeForItems = value.stream().mapToLong(inner -> FfiConverterTypeElement.INSTANCE.allocationSize(inner)).sum();
    return sizeForLength + sizeForItems;
  }

  @Override
  public void write(java.util.List<Element> value, java.nio.ByteBuffer buf) {
    buf.putInt(value.size());
    value.forEach(inner -> FfiConverterTypeElement.INSTANCE.write(inner, buf));
  }
}



