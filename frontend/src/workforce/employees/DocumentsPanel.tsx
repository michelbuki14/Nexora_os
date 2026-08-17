import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { errorMessage } from "../../api/client";
import { getDocumentUrl, listDocuments, uploadDocument } from "../../api/workforce";
import type { DocType } from "../../api/types";
import { usePermission } from "../../auth/usePermission";
import { FileUploader } from "../../components/FileUploader";
import { Modal } from "../../components/Modal";
import { useToast } from "../../components/Toast";
import { Alert, Button, Card, EmptyState, Field, Select } from "../../components/ui";
import { formatDate } from "../../lib/format";

const DOC_TYPES: { value: DocType; label: string }[] = [
  { value: "contract", label: "Contract" },
  { value: "id_document", label: "ID document" },
  { value: "certificate", label: "Certificate" },
  { value: "offer_letter", label: "Offer letter" },
  { value: "payslip", label: "Payslip" },
  { value: "other", label: "Other" },
];

const schema = z.object({ doc_type: z.string().min(1, "Document type is required") });
type FormValues = z.infer<typeof schema>;

function formatBytes(bytes: number | null | undefined): string {
  if (!bytes) return "—";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Document bytes are never rendered inline. Downloads use a short-lived
 * presigned GET URL (TTL ≤ 60s). Metadata only; the API stores bytes in MinIO.
 */
export function DocumentsPanel({ employeeUlid }: { employeeUlid: string }) {
  const canRead = usePermission("employee.documents.read");
  const canWrite = usePermission("employee.documents.write");
  const toast = useToast();
  const queryClient = useQueryClient();
  const [uploadOpen, setUploadOpen] = useState(false);
  const [file, setFile] = useState<File | null>(null);
  const [downloadError, setDownloadError] = useState<string | null>(null);

  const query = useQuery({
    queryKey: ["workforce", "employees", employeeUlid, "documents"],
    queryFn: () => listDocuments(employeeUlid, 1, 50),
    enabled: canRead,
  });

  const mutation = useMutation({
    mutationFn: async (values: FormValues) => {
      if (!file) throw new Error("No file selected");
      await uploadDocument(employeeUlid, {
        doc_type: values.doc_type as DocType,
        filename: file.name,
        mime_type: file.type || "application/octet-stream",
        size_bytes: file.size,
      }, file);
    },
    onSuccess: () => {
      toast.success("Document uploaded");
      setUploadOpen(false);
      setFile(null);
      queryClient.invalidateQueries({ queryKey: ["workforce", "employees", employeeUlid, "documents"] });
    },
    onError: (err) => toast.error(errorMessage(err)),
  });

  const { register, handleSubmit, reset } = useForm<FormValues>({ resolver: zodResolver(schema) });

  const download = async (docUlid: string, filename: string) => {
    setDownloadError(null);
    try {
      const { presigned_url } = await getDocumentUrl(employeeUlid, docUlid);
      // The URL is valid for ~60s. We open it directly; nothing is cached client-side.
      window.open(presigned_url, "_blank", "noopener");
      toast.info(`Download started for ${filename}`);
    } catch (err) {
      setDownloadError(errorMessage(err));
      toast.error(`Download failed: ${errorMessage(err)}`);
    }
  };

  if (!canRead) {
    return (
      <Card className="p-4">
        <p className="text-sm text-slate-500">
          Documents are restricted. You need the{" "}
          <code className="rounded bg-slate-100 px-1">employee.documents.read</code> permission
          to view documents.
        </p>
      </Card>
    );
  }

  return (
    <Card className="p-4">
      <div className="mb-3 flex items-center justify-between">
        <p className="text-sm font-medium text-slate-900">Documents</p>
        {canWrite && (
          <FileUploader
            label="Upload document"
            onSelect={(f) => {
              setFile(f);
              reset();
              setUploadOpen(true);
            }}
          />
        )}
      </div>

      {downloadError && <Alert kind="danger" >{downloadError}</Alert>}

      {query.isPending && <p className="text-sm text-slate-400">Loading…</p>}
      {query.isError && (
        <p className="text-sm text-red-600">Could not load documents: {errorMessage(query.error)}</p>
      )}
      {query.data && query.data.items.length === 0 && (
        <EmptyState
          title="No documents"
          description={canWrite ? "Upload an employment document." : "No documents have been uploaded."}
        />
      )}
      {query.data && query.data.items.length > 0 && (
        <table className="min-w-full divide-y divide-slate-200 text-sm">
          <thead className="bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
            <tr>
              <th className="px-3 py-2 font-medium">Filename</th>
              <th className="px-3 py-2 font-medium">Type</th>
              <th className="px-3 py-2 font-medium">Size</th>
              <th className="px-3 py-2 font-medium">Uploaded</th>
              <th className="px-3 py-2" />
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100">
            {query.data.items.map((doc) => (
              <tr key={doc.ulid} className="hover:bg-slate-50">
                <td className="px-3 py-2 text-slate-800">{doc.filename}</td>
                <td className="px-3 py-2 text-slate-600">{doc.doc_type}</td>
                <td className="px-3 py-2 text-slate-600">{formatBytes(doc.size_bytes)}</td>
                <td className="px-3 py-2 text-slate-600">{formatDate(doc.created_at)}</td>
                <td className="px-3 py-2 text-right">
                  {doc.is_active ? (
                    <Button variant="secondary" size="sm" onClick={() => download(doc.ulid, doc.filename)}>
                      Download
                    </Button>
                  ) : (
                    <span className="text-xs text-slate-400">Inactive</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <Modal open={uploadOpen} title="Upload document" onClose={() => setUploadOpen(false)}>
        <form onSubmit={handleSubmit((v) => mutation.mutate(v))} className="space-y-3">
          <Field label="Document type" htmlFor="doc-type">
            <Select id="doc-type" {...register("doc_type")}>
              <option value="">Select…</option>
              {DOC_TYPES.map((t) => (
                <option key={t.value} value={t.value}>{t.label}</option>
              ))}
            </Select>
          </Field>
          {file && (
            <p className="text-sm text-slate-600">
              <span className="font-medium">{file.name}</span> · {formatBytes(file.size)}
            </p>
          )}
          <p className="text-xs text-slate-400">
            The file is stored in object storage; AOS keeps only metadata and a
            content hash in the database.
          </p>
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="secondary" type="button" onClick={() => setUploadOpen(false)}>
              Cancel
            </Button>
            <Button type="submit" loading={mutation.isPending} disabled={!file}>
              Upload
            </Button>
          </div>
        </form>
      </Modal>
    </Card>
  );
}
