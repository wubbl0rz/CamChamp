using FFmpeg.AutoGen;

namespace backend;

public static class Extensions
{
  public static unsafe IReadOnlyList<AVStream> GetStreams(this AVFormatContext ctx, AVMediaType type)
  {
    var streams = new List<AVStream>();
    
    for (var i = 0; i < ctx.nb_streams; i++)
    {
      var stream = ctx.streams[i];

      if (stream->codecpar->codec_type == type)
      {
        streams.Add(*stream);
      }
    }

    return streams;
  }
}