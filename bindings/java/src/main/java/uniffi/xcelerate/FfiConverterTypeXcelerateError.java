package uniffi.xcelerate;


public enum FfiConverterTypeXcelerateError implements FfiConverterRustBuffer<XcelerateException> {
    INSTANCE;

    @Override
    public XcelerateException read(java.nio.ByteBuffer buf) {
        return switch(buf.getInt()) {
            case 1 -> new XcelerateException.WsException(FfiConverterString.INSTANCE.read(buf));
            case 2 -> new XcelerateException.SerdeException(FfiConverterString.INSTANCE.read(buf));
            case 3 -> new XcelerateException.CdpResponseException(FfiConverterString.INSTANCE.read(buf));
            case 4 -> new XcelerateException.HttpException(FfiConverterString.INSTANCE.read(buf));
            case 5 -> new XcelerateException.NotFound(FfiConverterString.INSTANCE.read(buf));
            case 6 -> new XcelerateException.InternalException(FfiConverterString.INSTANCE.read(buf));
            case 7 -> new XcelerateException.Unsupported(FfiConverterString.INSTANCE.read(buf));
            case 8 -> new XcelerateException.Plugin(FfiConverterString.INSTANCE.read(buf));
            default -> throw new java.lang.RuntimeException("invalid error enum value, something is very wrong!!");
        };
    }

    @Override
    public long allocationSize(XcelerateException value) {
        return 4L;
    }

    @Override
    public void write(XcelerateException value, java.nio.ByteBuffer buf) {
        switch(value) {
            case XcelerateException.WsException x -> {
                buf.putInt(1);
            }
            case XcelerateException.SerdeException x -> {
                buf.putInt(2);
            }
            case XcelerateException.CdpResponseException x -> {
                buf.putInt(3);
            }
            case XcelerateException.HttpException x -> {
                buf.putInt(4);
            }
            case XcelerateException.NotFound x -> {
                buf.putInt(5);
            }
            case XcelerateException.InternalException x -> {
                buf.putInt(6);
            }
            case XcelerateException.Unsupported x -> {
                buf.putInt(7);
            }
            case XcelerateException.Plugin x -> {
                buf.putInt(8);
            }
            default -> throw new java.lang.RuntimeException("invalid error enum value, something is very wrong!!");
        };
    }
}



