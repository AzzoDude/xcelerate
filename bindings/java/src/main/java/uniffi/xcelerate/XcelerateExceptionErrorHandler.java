package uniffi.xcelerate;


public class XcelerateExceptionErrorHandler implements UniffiRustCallStatusErrorHandler<XcelerateException> {
  @Override
  public XcelerateException lift(java.lang.foreign.MemorySegment errorBuf){
     return FfiConverterTypeXcelerateError.INSTANCE.lift(errorBuf);
  }
}

