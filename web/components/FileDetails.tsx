import {
  Clapperboard,
  Code,
  File, FileText,
  FolderArchive,
  Image,
  type LucideProps,
  Music,
  Presentation,
  Sheet
} from 'lucide-react'
import type {FileInfo} from "@/models.ts";
import {useTranslation} from "react-i18next";
import {
  isAudioFile,
  isCompressedFile,
  isExcelFile,
  isImageFile,
  isPdfFile, isPlainTextFile,
  isPowerPointFile, isSourceCodeFile,
  isVideoFile, isWordFile
} from "@/mime-type.ts";

interface FileDetailsProps {
  details: FileInfo
}

export default function FileDetails({ details }: FileDetailsProps) {
  const { t } = useTranslation()

  const formatSize = (size: number) => {
    const maxBytes = 1024
    const maxKb = 1024 * 1024
    const maxMb = 1024 * 1024 * 1024

    if (size < maxBytes) return `${size} B`
    if (size < maxKb) return `${Math.round(size / 1024)} KB`
    if (size < maxMb) return `${(size / (1024 * 1024)).toFixed(1)} MB`
    return `${(size / (1024 * 1024 * 1024)).toFixed(1)} GB`
  }

  const typeDescription = (mimeType: string) => {
    if (isVideoFile(mimeType)) return t('video')
    if (isAudioFile(mimeType)) return t('audio')
    if (isImageFile(mimeType)) return t('image')
    if (isCompressedFile(mimeType)) return t('compressed')
    if (isPdfFile(mimeType)) return t('pdf')
    if (isExcelFile(mimeType)) return t('spreadsheet')
    if (isPowerPointFile(mimeType)) return t('presentation')
    if (isSourceCodeFile(mimeType)) return t('source_code')
    if (isWordFile(mimeType)) return t('document')
    if (isPlainTextFile(mimeType)) return t('text')
    return t('file')
  }

  const TypeIcon = (props: { mimeType: string } & LucideProps) => {
    if (isVideoFile(props.mimeType)) return <Clapperboard {...props} />
    if (isAudioFile(props.mimeType)) return <Music {...props} />
    if (isImageFile(props.mimeType)) return <Image {...props} />
    if (isCompressedFile(props.mimeType)) return <FolderArchive {...props} />
    if (isPdfFile(props.mimeType)) return <FileText {...props} />
    if (isExcelFile(props.mimeType)) return <Sheet {...props} />
    if (isPowerPointFile(props.mimeType)) return <Presentation {...props} />
    if (isSourceCodeFile(props.mimeType)) return <Code {...props} />
    if (isWordFile(props.mimeType)) return <FileText {...props} />
    if (isPlainTextFile(props.mimeType)) return <FileText {...props} />
    return <File {...props} />
  }

  return (
    <div className="flex flex-col sm:flex-row">
      <div className="rounded-md flex size-20 p-1 items-center justify-center sm:mr-3 bg-muted mx-auto sm:mx-0">
        <TypeIcon mimeType={details.mime_type} className="size-10"/>
      </div>
      <div className="flex flex-col justify-between mt-3 py-1 sm:mt-0">
        <h1 className="text-2xl line-clamp-1 text-center sm:text-left">{details.name}</h1>
        <p className="text-lg line-clamp-1 text-center sm:text-left">{typeDescription(details.mime_type)} · {formatSize(details.size)}</p>
      </div>
    </div>
  )
}
