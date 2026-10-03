package uniffi.xcelerate;



public class XcelerateException extends java.lang.Exception {
    private XcelerateException(java.lang.String message) {
      super(message);
    }

    
    public static class WsException extends XcelerateException {
      public WsException(java.lang.String message) {
        super(message);
      }
    }
    
    public static class SerdeException extends XcelerateException {
      public SerdeException(java.lang.String message) {
        super(message);
      }
    }
    
    public static class CdpResponseException extends XcelerateException {
      public CdpResponseException(java.lang.String message) {
        super(message);
      }
    }
    
    public static class HttpException extends XcelerateException {
      public HttpException(java.lang.String message) {
        super(message);
      }
    }
    
    public static class NotFound extends XcelerateException {
      public NotFound(java.lang.String message) {
        super(message);
      }
    }
    
    public static class InternalException extends XcelerateException {
      public InternalException(java.lang.String message) {
        super(message);
      }
    }
    
    public static class Unsupported extends XcelerateException {
      public Unsupported(java.lang.String message) {
        super(message);
      }
    }
    
}

