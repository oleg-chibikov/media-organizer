using System;
using System.Drawing;
using System.IO;
using System.Text;
using System.Text.RegularExpressions;
using System.Windows.Forms;
using Microsoft.WindowsAPICodePack.Shell;

namespace MediaOrganizer
{
    partial class Program
    {
        static bool useDateFolders = false;
        static bool rename = false;
        static bool move = false;
        static bool recursive = false;
        static int fileCount = 0;
        static int counter = 0;
        static string rootDirectoryPath = "";


        //we init this once so that if the function is repeatedly called
        //it isn't stressing the garbage collector
        static readonly Regex ExifRegex = CreateExifRegex();

        [STAThread]
        static void Main()
        {
            var repeat = true;
            while (repeat)
            {
                Console.ForegroundColor = ConsoleColor.White;

                Console.WriteLine();
                Console.Write("Recursive (y/n): ");
                recursive = Console.ReadKey().KeyChar == 'y';
                Console.WriteLine();
                Console.WriteLine($"Recursive: {recursive}");

                Console.Write("Enter directory to rename files: ");
                while (string.IsNullOrWhiteSpace(rootDirectoryPath) || !Directory.Exists(rootDirectoryPath))
                {
                    using var fbd = new FolderBrowserDialog { AutoUpgradeEnabled = true };
                    if (fbd.ShowDialog() == DialogResult.OK)
                    {
                        rootDirectoryPath = fbd.SelectedPath;
                    }
                    else
                    {
                        return;
                    }
                }

                Console.WriteLine();
                Console.WriteLine("Calculating file count...");

                fileCount = recursive ? Directory.GetFiles(rootDirectoryPath, "*.*", SearchOption.AllDirectories).Length : Directory.GetFiles(rootDirectoryPath).Length;
                Console.WriteLine();
                Console.WriteLine($"{rootDirectoryPath}: {fileCount} files");

                Console.WriteLine();
                Console.Write("Move to date folders (y/n): ");
                move = Console.ReadKey().KeyChar == 'y';
                Console.WriteLine();
                Console.WriteLine($"Move to date folders: {move}");

                if (move)
                {
                    Console.WriteLine();
                    Console.Write("Use day folders (y/n): ");
                    useDateFolders = Console.ReadKey().KeyChar == 'y';
                    Console.WriteLine();
                    Console.WriteLine($"Use day folders: {useDateFolders}");
                }

                Console.WriteLine();
                Console.Write("Rename files (y/n): ");
                rename = Console.ReadKey().KeyChar == 'y';
                Console.WriteLine();
                Console.WriteLine($"Rename files: {rename}");

                RenameFilesInDirectory(rootDirectoryPath);

                Console.WriteLine();
                Console.Write("Done! Would you like to continue (y/n)?: ");
                repeat = Console.ReadKey().KeyChar == 'y';
                Console.WriteLine();
                Console.WriteLine($"Continue: {repeat}");
            }
        }

        static void RenameFilesInDirectory(string directoryPath)
        {
            Console.WriteLine($"Processing directory {directoryPath}");
            foreach (var filePath in Directory.EnumerateFiles(directoryPath))
            {
                RenameFile(filePath, directoryPath);
            }
            if (recursive)
            {
                foreach (var subDirectoryPath in Directory.EnumerateDirectories(directoryPath))
                {
                    RenameFilesInDirectory(subDirectoryPath);
                }
            }
        }

        static void RenameFile(string filePath, string directoryPath)
        {
            var newFilePath = GetNewFilePath(filePath, directoryPath);
            counter++;
            var percent = counter * 100 / fileCount;
            var progress = $"[{counter} / {fileCount} ({percent})%] {filePath}: ";

            if (newFilePath != null)
            {
                if (filePath == newFilePath)
                {
                    Console.ForegroundColor = ConsoleColor.White;
                    Console.WriteLine($"{progress}Already renamed");
                }
                else
                {
                    File.Move(filePath, newFilePath);
                    Console.ForegroundColor = ConsoleColor.Green;
                    Console.WriteLine($"{progress}Success");
                }
            }
            else
            {
                Console.ForegroundColor = ConsoleColor.Red;
                Console.WriteLine($"{progress}Cannot get file metadata");
            }
        }

        static string? GetNewFilePath(string filePath, string directoryPath)
        {
            DateTime? dateTaken;
            string? cameraModel = null;
            try
            {
                (dateTaken, cameraModel) = GetMetadataFromImage(filePath);
            }
            catch
            {
                Console.ForegroundColor = ConsoleColor.Red;
                dateTaken = GetMediaCreatedDate(filePath);
            }

            if (dateTaken != null)
            {
                var oldName = Path.GetFileNameWithoutExtension(filePath);
                string newName;
                if (rename)
                {
                    newName = $"{dateTaken.Value:yyyy-MM-dd HH-mm-ss}";

                    if (oldName.StartsWith(newName))
                    {
                        newName = oldName;
                    }
                    else
                    {
                        newName += $" ({oldName}){(cameraModel != null ? $" - {cameraModel}" : null)}";
                    }
                }
                else
                {
                    newName = oldName;
                }

                var newDirectoryPath = move ? GetHierarchicalDirectoryPath(rootDirectoryPath, dateTaken.Value) : directoryPath;

                var newFilePath = Path.Combine(newDirectoryPath, $"{newName}{Path.GetExtension(filePath)}");

                var duplicateNameMarker = 1;
                while (File.Exists(newFilePath))
                {
                    newFilePath = Path.Combine(newDirectoryPath,
                        $"{newName} ({++duplicateNameMarker}){Path.GetExtension(filePath)}");
                }

                return newFilePath;
            }

            return null;
        }

        static string GetHierarchicalDirectoryPath(string directoryPath, DateTime dateTaken)
        {
            var yearDirectoryPath = Path.Combine(directoryPath, dateTaken.Year.ToString());
            CreateDirectory(yearDirectoryPath);

            var monthDirectoryPath = Path.Combine(yearDirectoryPath, dateTaken.Month.ToString());
            CreateDirectory(monthDirectoryPath);
            if (useDateFolders)
            {
                var dayDirectoryPath = Path.Combine(monthDirectoryPath, dateTaken.Day.ToString());
                CreateDirectory(dayDirectoryPath);
                return dayDirectoryPath;
            }
            return monthDirectoryPath;
        }

        static void CreateDirectory(string path)
        {
            if (!Directory.Exists(path))
            {
                Directory.CreateDirectory(path);
            }
        }

        //retrieves the datetime WITHOUT loading the whole image
        static (DateTime, string?) GetMetadataFromImage(string path)
        {
            using var fs = new FileStream(path, FileMode.Open, FileAccess.Read);
            using var myImage = Image.FromStream(fs, false, false);
            var dateTakenBytes = myImage.GetPropertyItem(36867)?.Value ?? throw new InvalidOperationException();
            var dateTaken = ExifRegex.Replace(Encoding.UTF8.GetString(dateTakenBytes), "-", 2);

            string? cameraModel = null;
            var cameraModelBytes = myImage.GetPropertyItem(272)?.Value;
            if (cameraModelBytes != null)
            {
                cameraModel = Encoding.UTF8.GetString(cameraModelBytes);
                cameraModel = cameraModel[..^1];
            }

            return (DateTime.Parse(dateTaken), cameraModel);
        }

        static DateTime? GetMediaCreatedDate(string path)
        {
            var shell = ShellObject.FromParsingName(path);

            var data = shell.Properties.System.Media.DateEncoded;
            return data.Value;
        }

        [GeneratedRegex(":", RegexOptions.Compiled)]
        private static partial Regex CreateExifRegex();
    }
}
