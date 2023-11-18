using System.Runtime.InteropServices;

namespace backend;

public struct ConnectionResult() : IDisposable
{
  public uint ClientId;
  private readonly IntPtr _offer = IntPtr.Zero;

  public string? GetOffer()
  {
    var str = Marshal.PtrToStringUTF8(_offer);
    return str;
  }

  public void Dispose()
  {
    InteropGo.freeStr(_offer);
  }
}

public unsafe class InteropGo
{
  [DllImport("webrtc_linux", EntryPoint = "freeStr")]
  public static extern void freeStr(IntPtr ptr);
  
  [DllImport("webrtc_linux", EntryPoint = "initNetwork")]
  public static extern void initNetwork();
  
  [DllImport("webrtc_linux", EntryPoint = "createTrack")]
  public static extern void createTrack();
  
  [DllImport("webrtc_linux", EntryPoint = "createConnection")]
  public static extern ConnectionResult createConnection();
  
  [DllImport("webrtc_linux", EntryPoint = "setAnswer")]
  public static extern ConnectionResult setAnswer(int id, [MarshalAs(UnmanagedType.LPUTF8Str)]string sdp);
}

public unsafe class InteropRust
{
  [DllImport("librtc", EntryPoint = "init")]
  public static extern void init();
}
